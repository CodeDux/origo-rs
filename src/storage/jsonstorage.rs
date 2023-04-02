use super::Storage;
use crate::{Command, CommandRestoreFn};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Seek, Write},
    path::Path,
};

pub struct JsonStorage {
    journal_file: File,
}

impl JsonStorage {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        JsonStorage {
            journal_file: match path.as_ref().exists() {
                true => File::options().read(true).write(true).open(path).unwrap(),
                false => {
                    if let Some(directory) = path.as_ref().parent() {
                        std::fs::create_dir_all(directory)
                            .expect("Failed to create directory structure for ");
                    }
                    File::create(path).expect("Failed to create journal-file")
                }
            },
        }
    }
}

impl Storage for JsonStorage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(&mut self, name: &str, command: &T) {
        self.journal_file
            .write_all(name.as_bytes())
            .expect("Failed to write identifier");
        serde_json::to_writer(&self.journal_file, command).expect("Failed to write payload");
    }

    fn commit_command(&mut self) {
        self.journal_file
            .write_all(&[b'\n'])
            .expect("Failed to write ending");
        self.journal_file.flush().expect("Failed to flush");
    }

    fn restore<TModel>(
        &mut self,
        model: &mut TModel,
        restore_fns: &HashMap<String, CommandRestoreFn<JsonStorage, TModel>>,
    ) {
        let mut reader = BufReader::new(&self.journal_file);
        let mut buffer = vec![0u8; 0];

        let file_len = self.journal_file.metadata().unwrap().len();

        let mut entries_count = 0;
        loop {
            match reader.read_until(b'\n', &mut buffer) {
                Ok(bytes) if bytes > 0 => {}
                _ => break,
            };

            // IF we don't end on `\n` that means that it was a failed command
            // we need to:
            // - Ignore the current row
            // - Shrink the file to where the row started
            match buffer.last() {
                Some(&b'\n') => {}
                Some(_) if file_len == reader.stream_position().unwrap_or(0) => {
                    println!("Removing corrupt entry");
                    self.journal_file
                        .set_len(file_len - buffer.len() as u64)
                        .expect("Couldn't shrink journal");
                    break;
                }
                _ => panic!("Corrupt journal"),
            }

            let command_name_length = buffer.iter().position(|c| c == &b'{').unwrap();
            let (command_name_bytes, command_data) = buffer.split_at(command_name_length);

            let command_name = std::str::from_utf8(command_name_bytes).unwrap();
            let restore_fn = restore_fns.get(command_name).unwrap();

            restore_fn(self, command_data, model);
            buffer.clear();
            entries_count += 1;
        }

        println!("Loaded {entries_count} events");
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
