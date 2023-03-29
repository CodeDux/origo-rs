use super::Storage;
use crate::Command;
use std::collections::HashMap;

/// This does nothing with commands, truly in-memory mode with no persistance state
pub struct NoopStorage;

impl Storage for NoopStorage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(&mut self, _command: &T) {}

    fn commit_command(&mut self) {}

    fn restore<TModel>(
        &mut self,
        _model: &mut TModel,
        _commands: &HashMap<String, Box<dyn Fn(&[u8], &mut TModel)>>,
    ) {
    }
}
