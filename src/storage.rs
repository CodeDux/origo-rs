use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use crate::Command;

pub struct JsonStorage {
    journal_file: File,
    buffer: Vec<u8>,
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
            buffer: Vec::with_capacity(0),
        }
    }

    pub fn prepare_command<'de, TModel, T: Command<'de, TModel>>(&mut self, command: &T) {
        let identifier = T::identifier();

        self.journal_file
            .write_all(identifier.as_bytes())
            .expect("Failed to write identifier");
        serde_json::to_writer(&self.journal_file, command).expect("Failed to write payload");
    }

    pub fn commit_command(&mut self) {
        self.journal_file
            .write_all(&[b'\n'])
            .expect("Failed to write ending");
        self.journal_file.flush().expect("Failed to flush");
    }

    pub fn restore<TModel>(
        &mut self,
        model: &mut TModel,
        commands: &HashMap<String, Box<dyn Fn(&[u8], &mut TModel) -> ()>>,
    ) {
        let mut reader = BufReader::new(&self.journal_file);
        let mut buffer = &mut self.buffer;

        let file_len = self.journal_file.metadata().unwrap().len();

        let mut entries_count = 0;
        loop {
            buffer.clear();

            // Read name of command to restore
            let command_name = match reader.read_until(b'{', &mut buffer) {
                Ok(bytes) if bytes > 0 => std::str::from_utf8(&buffer[..bytes - 1]).expect("msg"),
                _ => break,
            };

            // Find the command
            let command = commands.get(command_name).unwrap();
            let start = buffer.len() - 1; // The index for including the `{` in the `ComandName{`

            // Read the rest of the line until we get to the end `\n`
            let command_data = match reader.read_until(b'\n', &mut buffer) {
                Ok(bytes) if bytes > 0 => &buffer[start..],
                _ => break,
            };

            // IF we don't end on `\n` that means that it was a failed command
            // we need to:
            // - Ignore the current row
            // - Shrink the file to where the row started
            match command_data.last() {
                Some(byte) if byte == &b'\n' => {}
                _ => {
                    self.journal_file
                        .set_len(file_len - buffer.len() as u64)
                        .unwrap();
                    break;
                }
            }

            command(command_data, model);
            entries_count += 1;
        }

        println!("Loaded {entries_count} events");
    }

    pub fn writer(&self) -> &File {
        &self.journal_file
    }
}
