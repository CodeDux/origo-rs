use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc};

use crate::storage::JsonStorage;

pub trait Command<'de, TModel>: Serialize + Deserialize<'de> {
    fn execute(&self, model: &mut TModel);
    fn identifier() -> &'static str;
}

pub struct Engine<TModel: Default> {
    model: Arc<RwLock<TModel>>,
    storage: Arc<Mutex<JsonStorage>>,
}

impl<TModel: Default> Engine<TModel> {
    /// Execute the given command against the current model
    ///
    /// Commands execute in exclusive mode,
    /// meaning that no other writes OR queries will happen until the command finishes (The model is ReadWriteLocked)
    ///
    /// Before executing the command it's written to the journal
    pub fn execute<'de, T: Command<'de, TModel>>(&self, command: &T) {
        let mut storage = self.storage.lock();
        let mut model = self.model.write();
        storage.prepare_command::<TModel, T>(command);
        command.execute(&mut model);
        storage.commit_command();
    }

    /// Execute the given query against the current model
    ///
    /// Multiple queries can execute against the model at the same time
    /// but no writes will happen during queries (The model is ReadWriteLocked)
    pub fn query<R, F: FnOnce(&TModel) -> R>(&self, query: F) -> R {
        let model = self.model.read();
        query(&model)
    }
}

impl<TModel: Default> Clone for Engine<TModel> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            storage: self.storage.clone(),
        }
    }
}

pub struct EngineBuilder<TModel: Default> {
    model: TModel,
    storage: JsonStorage,
    commands: HashMap<String, Box<dyn Fn(&[u8], &mut TModel) -> ()>>,
}

impl<TModel: Default> EngineBuilder<TModel> {
    pub fn new<T: AsRef<Path>>(path: T) -> EngineBuilder<TModel> {
        EngineBuilder {
            model: Default::default(),
            storage: JsonStorage::new(path),
            commands: HashMap::new(),
        }
    }

    pub fn register_command<'a, T>(mut self, f: Box<dyn Fn(&[u8], &mut TModel)>) -> Self
    where
        T: Command<'a, TModel>,
    {
        let name = T::identifier();
        assert!(!name.contains('{'));
        println!("Adding: {}", name);

        self.commands.insert(name.to_string(), f);
        self
    }

    pub fn build(mut self) -> Engine<TModel> {
        self.storage
            .restore::<TModel>(&mut self.model, &self.commands);

        Engine {
            model: Arc::new(RwLock::new(self.model)),
            storage: Arc::new(Mutex::new(self.storage)),
        }
    }
}
