use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use serde::{de::DeserializeOwned, Serialize};

const BUF_WRITER_CAPACITY: usize = 32 * 1024;

/// # Panics
///
/// Panics if file can't be opened or the content can't be deserialized.
pub fn read<TModel: Default + DeserializeOwned>(snapshot_file: &PathBuf) -> TModel {
    let instant = Instant::now();
    let snapshot_file = File::options()
        .read(true)
        .open(snapshot_file)
        .expect("Failed to open snapshot");

    let snapshot_reader = BufReader::with_capacity(BUF_WRITER_CAPACITY, snapshot_file);

    let model = serde_json::from_reader(snapshot_reader).unwrap();
    log::debug!("Loaded snapshot in: {}ms", instant.elapsed().as_millis());
    model
}

pub fn write<TModel: Serialize>(snapshot_path: &PathBuf, model: &TModel) -> Result<(), ()> {
    let snapshot_file = File::options().write(true).create(true).open(snapshot_path);

    match snapshot_file {
        Ok(file) => {
            let instant = Instant::now();
            let mut writer = BufWriter::with_capacity(BUF_WRITER_CAPACITY, file);
            match serde_json::to_writer_pretty(&mut writer, model) {
                Ok(_) => {
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
