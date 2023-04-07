use serde::Serialize;

use crate::{
    engine::{Command, CommandRestoreFn},
    storage::{CommitResult, Storage},
};

use std::collections::HashMap;

/// This does nothing with commands, truly in-memory mode with no persistance state or snapshots
pub struct NoopStorage;

impl Storage for NoopStorage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(
        &mut self,
        _command_name: &str,
        _command: &T,
    ) {
    }

    fn commit_command(&mut self) -> CommitResult {
        Ok(-1)
    }

    fn snapshot<TModel: Serialize>(&mut self, _model: &TModel) {}

    fn restore<TModel: Default + serde::de::DeserializeOwned>(
        &mut self,
        _restore_fns: &HashMap<String, CommandRestoreFn<Self, TModel>>,
    ) -> TModel {
        TModel::default()
    }

    fn restore_command<'de, TModel, T: Command<'de, TModel>>(
        &self,
        _data: &'de [u8],
        _model: &mut TModel,
    ) {
    }
}
