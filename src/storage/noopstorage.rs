use super::Storage;
use crate::{Command, CommandRestoreFn};
use std::collections::HashMap;

/// This does nothing with commands, truly in-memory mode with no persistance state
pub struct NoopStorage;

impl Storage for NoopStorage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(
        &mut self,
        _command_name: &str,
        _command: &T,
    ) {
    }

    fn commit_command(&mut self) {}

    fn restore<TModel>(
        &mut self,
        _model: &mut TModel,
        _restore_fns: &HashMap<String, CommandRestoreFn<NoopStorage, TModel>>,
    ) {
    }
    fn restore_command<'de, TModel, T: Command<'de, TModel>>(
        &self,
        _data: &'de [u8],
        _model: &mut TModel,
    ) {
    }
}
