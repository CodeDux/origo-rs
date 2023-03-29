mod noopstorage;
pub use noopstorage::NoopStorage;

mod jsonstorage;
pub use jsonstorage::JsonStorage;

use crate::Command;
use std::collections::HashMap;

pub trait Storage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(&mut self, command: &T);
    fn commit_command(&mut self);
    fn restore<TModel>(
        &mut self,
        model: &mut TModel,
        commands: &HashMap<String, Box<dyn Fn(&[u8], &mut TModel) -> ()>>,
    );
}
