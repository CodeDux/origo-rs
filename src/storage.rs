mod noopstorage;
pub use noopstorage::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use crate::Command;

pub trait Storage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(&mut self, command: &T);
    fn commit_command(&mut self);
    fn restore<TModel>(
        &mut self,
        model: &mut TModel,
        commands: &HashMap<String, Box<dyn Fn(&[u8], &mut TModel) -> ()>>,
    );
}

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
                            .expect("Failed to create directory structure");
                    }
                    File::create(path).unwrap()
                }
            },
        }
    }
}

impl Storage for JsonStorage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(&mut self, command: &T) {
        let identifier = T::identifier();

        self.journal_file
            .write_all(identifier.as_bytes())
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
        commands: &HashMap<String, Box<dyn Fn(&[u8], &mut TModel) -> ()>>,
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
                Some(byte) if byte == &b'\n' => {}
                _ => {
                    self.journal_file
                        .set_len(file_len - buffer.len() as u64)
                        .unwrap();
                    break;
                }
            }

            let command_name_length = buffer.iter().position(|c| c == &b'{').unwrap();
            let (command_name_bytes, command_data) = buffer.split_at(command_name_length);

            let command_name = std::str::from_utf8(&command_name_bytes).unwrap();
            let command = commands.get(command_name).unwrap();

            command(command_data, model);
            buffer.clear();
            entries_count += 1;
        }

        println!("Loaded {entries_count} events");
    }
}
