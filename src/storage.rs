mod noopstorage;
pub use noopstorage::NoopStorage;

mod jsonstorage;
pub use jsonstorage::JsonStorage;

use crate::{Command, CommandRestoreFn};
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
        restore_fns: &HashMap<String, CommandRestoreFn<Self, TModel>>,
    );
    fn restore_command<'de, TModel, T: Command<'de, TModel>>(
        &self,
        data: &'de [u8],
        model: &mut TModel,
    );
}
