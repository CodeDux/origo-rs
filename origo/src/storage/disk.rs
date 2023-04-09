use crate::{
    engine::{Command, CommandRestoreFn},
    storage::Storage,
};

use bincode::{config::Configuration, Decode};

use core::panic;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, ErrorKind, Read, Seek, Write},
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
    time::Instant,
};

pub struct DiskStorage {
    journal_file: File,
    directory: std::path::PathBuf,
    buf_writer: BufWriter<File>,
    command_count_current: u64,
    commit_buffer: Vec<u8>,
}

const BUFFER_CAPACITY: usize = 32 * 1024;
static BINCODE_CONFIG: Configuration = bincode::config::standard();

impl DiskStorage {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        let journal_file = match path.as_ref().exists() {
            true => {
                let file = File::options()
                    .read(true)
                    .write(true)
                    .open(path.as_ref())
                    .expect("Failed to open journal");
                file
            }
            false => {
                if let Some(directory) = path.as_ref().parent() {
                    std::fs::create_dir_all(directory)
                        .expect("Failed to create directory structure");
                }
                let mut file = File::create(path.as_ref()).expect("Failed to create journal-file");
                file.write_all(&[0u8; 8])
                    .expect("Failed to initialize journal");
                file
            }
        };

        DiskStorage {
            directory: path.as_ref().parent().unwrap_or(Path::new("")).to_owned(),
            buf_writer: BufWriter::with_capacity(
                BUFFER_CAPACITY,
                journal_file.try_clone().unwrap(),
            ),
            journal_file,
            command_count_current: 0,
            commit_buffer: Vec::<u8>::with_capacity(BUFFER_CAPACITY),
        }
    }

    fn replay_journal<TModel: Default + Decode>(
        &mut self,
        model: &mut TModel,
        restore_fns: &HashMap<String, CommandRestoreFn<TModel>>,
    ) {
        let file_len = self.journal_file.metadata().unwrap().len();

        if file_len <= 8 {
            return;
        }

        let mut entries = [0u8; 8];
        self.journal_file
            .read_exact(&mut entries)
            .expect("failed to read entries count");

        let entries_count = u64::from_le_bytes(entries);
        log::debug!("Loading {} events from journal", &entries_count);

        let mut reader = BufReader::new(&self.journal_file);

        let mut len_header = [0u8; 8];
        let mut name_len_header = [0u8; 8];
        let mut data = vec![0u8; BUFFER_CAPACITY];

        for i in 0..entries_count {
            match reader.read_exact(&mut len_header) {
                Ok(_) => {
                    let data_len = u64::from_le_bytes(len_header) as usize;
                    data.resize(data_len, 0);
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => panic!("{:?}", i),
                Err(e) => panic!("Journal restore failed, {:?}", e),
            }

            let command_name_length = match reader.read_exact(&mut name_len_header) {
                Ok(_) => u64::from_le_bytes(name_len_header) as usize,
                Err(e) => panic!("Journal restore failed, {:?}", e),
            };

            match reader.read_exact(&mut data) {
                Ok(_) => {
                    let command_name = std::str::from_utf8(&data[..command_name_length])
                        .unwrap_or_else(|_| {
                            panic!(
                                "Failed to parse command({}) name bytes to utf8",
                                self.command_count_current
                            );
                        });

                    let restore_fn = restore_fns.get(command_name).unwrap_or_else(|| {
                        panic!("No restore registered for command {}", command_name)
                    });

                    match restore_fn(&data[command_name_length..], model, BINCODE_CONFIG) {
                        true => self.command_count_current += 1,
                        false => panic!(
                            "Corrupt journal, failed to restore {}({})",
                            command_name, self.command_count_current
                        ),
                    }
                }
                Err(e) => panic!("Journal restore failed, {:?}", e),
            }
        }
    }
}

///
///
///
impl Storage for DiskStorage {
    fn prepare<TModel, T: Command<TModel>>(&mut self, name: &str, command: &T) {
        self.commit_buffer.clear();
        self.commit_buffer.write_all(&[0u8; 8]).expect(""); // reserve space for total length header

        let name_bytes = name.as_bytes();
        self.commit_buffer
            .write_all(&(name_bytes.len() as u64).to_le_bytes())
            .expect("Failed to write name len");

        self.commit_buffer
            .write_all(name_bytes)
            .expect("Failed to write identifier");

        let mut len = name_bytes.len();

        len += bincode::encode_into_std_write(command, &mut self.commit_buffer, BINCODE_CONFIG)
            .expect("Failed to serialize command to bytes");

        self.commit_buffer[..8].copy_from_slice(&(len as u64).to_le_bytes());
    }

    fn commit(&mut self) -> u64 {
        self.buf_writer
            .write_all(&self.commit_buffer)
            .expect("Failed to commit command");

        self.buf_writer.flush().expect("Failed to flush commit");
        self.journal_file.sync_all().expect("Sync to disk failed");
        self.command_count_current += 1;

        self.journal_file
            .write_all_at(&u64::to_le_bytes(self.command_count_current), 0)
            .expect("msg");

        self.command_count_current
    }

    fn snapshot<TModel: bincode::Encode>(&mut self, model: &TModel) {
        let snapshot_path = self.directory.join("snap.origors");
        match snapshot_write(&snapshot_path, model) {
            Ok(_) => {
                self.journal_file
                    .set_len(0)
                    .expect("Failed to reset journal length");
                self.journal_file
                    .rewind()
                    .expect("Failed to rewind journal");
                self.journal_file
                    .write_all(&[0u8; 8])
                    .expect("Failed to reset journal header");
                self.journal_file
                    .flush()
                    .expect("Failed to flush journal to disk");
                self.journal_file
                    .sync_all()
                    .expect("Failed to sync journal to disk");

                self.buf_writer
                    .seek(std::io::SeekFrom::Start(8))
                    .expect("Failed to reset writer");
                self.command_count_current = 0;
            }
            Err(_) => {
                panic!("Snapshot write to disk failed");
            }
        }
    }

    fn restore<TModel: Default + bincode::Decode>(
        &mut self,
        restore_fns: &HashMap<String, CommandRestoreFn<TModel>>,
    ) -> TModel {
        let snapshot_path = self.directory.join("snap.origors");

        let mut model = match snapshot_path.exists() {
            true => snapshot_read(&snapshot_path),
            false => TModel::default(),
        };

        let instant = Instant::now();
        self.replay_journal(&mut model, restore_fns);

        log::debug!(
            "Loaded {} events from journal in {}ms",
            &self.command_count_current,
            instant.elapsed().as_millis()
        );

        model
    }
}

/// # Panics
///
/// Panics if file can't be opened or the content can't be deserialized.
fn snapshot_read<TModel: Default + bincode::Decode>(snapshot_file: &PathBuf) -> TModel {
    let instant = Instant::now();
    let snapshot_file = File::options()
        .read(true)
        .open(snapshot_file)
        .expect("Failed to open snapshot");

    let mut snapshot_reader = BufReader::with_capacity(BUFFER_CAPACITY, snapshot_file);

    let model = bincode::decode_from_std_read(&mut snapshot_reader, BINCODE_CONFIG).unwrap();
    log::debug!("Loaded snapshot in {}ms", instant.elapsed().as_millis());
    model
}

fn snapshot_write<TModel: bincode::Encode>(
    snapshot_path: &PathBuf,
    model: &TModel,
) -> Result<(), ()> {
    let snapshot_file = File::options().write(true).create(true).open(snapshot_path);

    match snapshot_file {
        Ok(file) => {
            let mut writer = BufWriter::with_capacity(BUFFER_CAPACITY, &file);

            let instant = Instant::now();

            match bincode::encode_into_std_write(model, &mut writer, BINCODE_CONFIG) {
                Ok(_) => {
                    file.sync_all().expect("Sync to disk failed");
                    log::debug!("Snapshot created in {}ms", instant.elapsed().as_millis());
                    Ok(())
                }
                Err(_) => {
                    log::error!("Snapshot write to disk failed");
                    Err(())
                }
            }
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => {
                log::error!("Snapshot already exists");
                Err(())
            }
            _ => Err(()),
        },
    }
}
