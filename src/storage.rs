mod noopstorage;
pub use noopstorage::NoopStorage;

mod jsonstorage;
pub use jsonstorage::JsonStorage;

use crate::{Command, CommandExecutor};
use std::collections::HashMap;

pub trait Storage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(
        &mut self,
        command_name: &str,
        command: &T,
    );
    fn commit_command(&mut self);
    fn restore<TModel>(
        &mut self,
        model: &mut TModel,
        commands: &HashMap<String, CommandExecutor<Self, TModel>>,
    );
    fn deserialize<'de, TModel, T: Command<'de, TModel>>(
        &self,
        data: &'de [u8],
        model: &mut TModel,
    );
}
