mod snapshot;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    engine::{Command, CommandRestoreFn},
    storage::{CommitResult, Storage},
};

use core::panic;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Seek, Write},
    path::Path,
};

pub struct JsonStorage {
    journal_file: File,
    directory: std::path::PathBuf,
    buf_writer: BufWriter<File>,
    command_count_current: i64,
    command_count_max: i64,
}

const BUF_WRITER_CAPACITY: usize = 32 * 1024;

impl JsonStorage {
    pub fn new<T: AsRef<Path>>(path: T, command_count_max: u32) -> Self {
        let journal_file = match path.as_ref().exists() {
            true => File::options()
                .read(true)
                .write(true)
                .open(path.as_ref())
                .unwrap(),
            false => {
                if let Some(directory) = path.as_ref().parent() {
                    std::fs::create_dir_all(directory)
                        .expect("Failed to create directory structure for ");
                }
                File::create(path.as_ref()).expect("Failed to create journal-file")
            }
        };

        JsonStorage {
            directory: path.as_ref().parent().unwrap_or(Path::new("")).to_owned(),
            buf_writer: BufWriter::with_capacity(
                BUF_WRITER_CAPACITY,
                journal_file.try_clone().unwrap(),
            ),
            journal_file,
            command_count_current: 0,
            command_count_max: command_count_max as i64,
        }
    }
}

impl Storage for JsonStorage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(&mut self, name: &str, command: &T) {
        self.buf_writer
            .write_all(name.as_bytes())
            .expect("Failed to write identifier");
        serde_json::to_writer(&mut self.buf_writer, command).expect("Failed to write payload");
    }

    fn commit_command(&mut self) -> CommitResult {
        self.buf_writer
            .write_all(&[b'\n'])
            .expect("Failed to write ending");
        self.buf_writer.flush().expect("Failed to flush");

        self.command_count_current += 1;

        Ok(self.command_count_max - self.command_count_current)
    }

    fn snapshot<TModel: Serialize>(&mut self, model: &TModel) {
        let snapshot_path = self.directory.join("snap.origors");
        match snapshot::write(&snapshot_path, model) {
            Ok(_) => {
                self.journal_file.set_len(0).unwrap();
                self.buf_writer.rewind().unwrap();
                self.command_count_current = 0;
            }
            Err(_) => {
                panic!("Snapshot write to disk failed");
            }
        }
    }

    fn restore<TModel: Default + DeserializeOwned>(
        &mut self,
        restore_fns: &HashMap<String, CommandRestoreFn<JsonStorage, TModel>>,
    ) -> TModel {
        let snapshot_path = self.directory.join("snap.origors");

        let mut model = match snapshot_path.exists() {
            true => snapshot::read(&snapshot_path),
            false => TModel::default(),
        };

        let file_len = self.journal_file.metadata().unwrap().len();

        if file_len == 0 {
            return model;
        }

        let mut reader = BufReader::new(&self.journal_file);
        let mut buffer = Vec::<u8>::new();
        let mut total_bytes_read = 0_u64;

        loop {
            match reader.read_until(b'\n', &mut buffer) {
                Ok(bytes) if bytes > 0 => total_bytes_read += bytes as u64,
                _ => break,
            };

            // IF we don't end on `\n` that means that it was a failed command
            // we need to:
            // - Ignore the current row
            // - Shrink the file to where the row started
            match buffer.last() {
                Some(&b'\n') => {
                    let command_name_length = buffer.iter().position(|c| c == &b'{').unwrap();
                    let (command_name_bytes, command_data) = buffer.split_at(command_name_length);

                    let command_name = std::str::from_utf8(command_name_bytes).unwrap();
                    let restore_fn = restore_fns.get(command_name).unwrap();

                    restore_fn(self, command_data, &mut model);
                    buffer.clear();
                    self.command_count_current += 1;
                }
                Some(_) if file_len == total_bytes_read => {
                    log::warn!("Removing corrupt entry");
                    self.journal_file
                        .set_len(file_len - buffer.len() as u64)
                        .expect("Couldn't shrink journal");
                    break;
                }
                _ => panic!("Corrupt journal"),
            }
        }

        log::debug!("Loaded {} events", &self.command_count_current);
        model
    }

    fn restore_command<'de, TModel, T: Command<'de, TModel>>(
        &self,
        data: &'de [u8],
        model: &mut TModel,
    ) {
        let command = serde_json::from_slice::<T>(data).unwrap();
        command.execute(model);
    }
}
