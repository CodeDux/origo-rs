use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::storage::Storage;

pub type CommandExecutor<TModel> = Box<dyn Fn(&[u8], &mut TModel)>;

pub trait Command<'de, TModel>: Serialize + Deserialize<'de> {
    fn execute(&self, model: &mut TModel);
    fn identifier() -> &'static str;
}

pub struct Engine<TModel: Default, TStorage> {
    inner: Arc<RwLock<(TModel, TStorage)>>,
}

impl<TModel: Default, TStorage: Storage> Engine<TModel, TStorage> {
    /// Execute the given command against the current model
    ///
    /// Commands execute in exclusive mode,
    /// meaning that no other writes OR queries will happen until the command finishes (The model is ReadWriteLocked)
    ///
    /// Before executing the command it's written to the journal
    pub fn execute<'de, T: Command<'de, TModel>>(&self, command: &T) {
        let mut inner = self.inner.write();
        inner.1.prepare_command::<TModel, T>(command);
        command.execute(&mut inner.0);
        inner.1.commit_command();
    }

    /// Execute the given query against the current model
    ///
    /// Multiple queries can execute against the model at the same time
    /// but no writes will happen during queries (The model is ReadWriteLocked)
    pub fn query<R, F: FnOnce(&TModel) -> R>(&self, query: F) -> R {
        let inner = self.inner.read();
        query(&inner.0)
    }
}

impl<TModel: Default, TStorage> Clone for Engine<TModel, TStorage> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct EngineBuilder<TModel: Default, TStorage> {
    model: TModel,
    storage: TStorage,
    commands: HashMap<String, CommandExecutor<TModel>>,
}

impl<TModel: Default, TStorage: Storage> EngineBuilder<TModel, TStorage> {
    pub fn new(model: TModel, storage: TStorage) -> EngineBuilder<TModel, TStorage> {
        EngineBuilder {
            model,
            storage,
            commands: HashMap::new(),
        }
    }

    pub fn register_command<'a, T>(mut self, f: CommandExecutor<TModel>) -> Self
    where
        T: Command<'a, TModel>,
    {
        let name = T::identifier();
        assert!(!name.contains('{'));
        println!("Adding: {}", name);

        self.commands.insert(name.to_string(), f);
        self
    }

    pub fn build(mut self) -> Engine<TModel, TStorage> {
        self.storage.restore(&mut self.model, &self.commands);

        Engine {
            inner: Arc::new(RwLock::new((self.model, self.storage))),
        }
    }
}
