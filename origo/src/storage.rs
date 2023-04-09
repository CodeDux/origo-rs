mod disk;
pub use disk::DiskStorage;

mod noop;
pub use noop::NoopStorage;

use crate::engine::{Command, CommandRestoreFn};
use std::collections::HashMap;

pub trait Storage {
    fn prepare<TModel, T: Command<TModel>>(&mut self, command_name: &str, command: &T);

    fn commit(&mut self) -> u64;

    fn snapshot<TModel: bincode::Encode>(&mut self, model: &TModel);

    fn restore<TModel: Default + bincode::Decode>(
        &mut self,
        restore_fns: &HashMap<String, CommandRestoreFn<TModel>>,
    ) -> TModel;
}
