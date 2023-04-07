mod noopstorage;
pub use noopstorage::NoopStorage;

mod jsonstorage;
pub use jsonstorage::JsonStorage;

use serde::{de::DeserializeOwned, Serialize};

use crate::engine::{Command, CommandRestoreFn};
use std::collections::HashMap;

pub type CommitResult = Result<i64, CommitError>;

pub struct CommitError;

pub trait Storage {
    fn prepare_command<'de, TModel, T: Command<'de, TModel>>(
        &mut self,
        command_name: &str,
        command: &T,
    );

    fn commit_command(&mut self) -> CommitResult;

    fn snapshot<TModel: Serialize>(&mut self, model: &TModel);

    fn restore<TModel: Default + DeserializeOwned>(
        &mut self,
        restore_fns: &HashMap<String, CommandRestoreFn<Self, TModel>>,
    ) -> TModel;

    fn restore_command<'de, TModel, T: Command<'de, TModel>>(
        &self,
        data: &'de [u8],
        model: &mut TModel,
    );
}

// #[cfg(test)]
// mod tests {
//     use super::TrieNode;

//     #[test]
//     fn add_works() {
//         let word = "Test";
//         let root = TrieNode {
//             character: b'a',
//             children: Vec::new(),
//         };
//     }
//}
