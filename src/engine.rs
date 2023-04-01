use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, collections::HashMap, sync::Arc};

use crate::storage::Storage;

pub type CommandExecutor<TStorage, TModel> = Box<dyn Fn(&TStorage, &[u8], &mut TModel)>;

pub trait Command<'de, TModel>: Serialize + Deserialize<'de> {
    fn execute(&self, model: &mut TModel);
}

pub struct Engine<TModel, TStorage> {
    model: Arc<RwLock<TModel>>,
    storage: Arc<Mutex<TStorage>>,
    commands: Arc<HashMap<TypeId, String>>,
}

impl<TModel, TStorage: Storage> Engine<TModel, TStorage> {
    /// Execute the given command against the current model
    ///
    /// Commands execute in exclusive mode,
    /// meaning that no other writes OR queries will happen until the command finishes
    /// (The model is ReadWriteLocked)
    ///
    /// Before executing the command it's written to the journal
    pub fn execute<'de, T>(&self, command: &T)
    where
        T: Command<'de, TModel> + 'static,
    {
        let command_name: &str = self
            .commands
            .get(&TypeId::of::<T>())
            .expect("Couldn't find command_name");

        // We lock storage before the model so we can allow queries during the possible storage IO
        // This is the reason for storing `storage` and `model` in separate locks
        let mut storage = self.storage.lock();
        storage.prepare_command::<TModel, T>(command_name, command);

        // Here we lock the model so no queries can happen before the new state is applied and
        // commited to disk.
        //
        // We also keep holding the lock on storage since we don't know if we are "safe" until
        // we commited/flushed the command (with commit indicator `\n` in the end)
        let mut model = self.model.write();
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

impl<TModel, TStorage> Clone for Engine<TModel, TStorage> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            storage: self.storage.clone(),
            commands: self.commands.clone(),
        }
    }
}

pub struct EngineBuilder<TModel, TStorage> {
    model: TModel,
    storage: TStorage,
    commands: HashMap<String, CommandExecutor<TStorage, TModel>>,
    command_names_by_id: HashMap<TypeId, String>,
}

impl<TModel, TStorage: Storage> EngineBuilder<TModel, TStorage> {
    pub fn new(model: TModel, storage: TStorage) -> EngineBuilder<TModel, TStorage> {
        EngineBuilder {
            model,
            storage,
            commands: HashMap::new(),
            command_names_by_id: HashMap::new(),
        }
    }

    pub fn register_command<'a, T>(
        mut self,
        name: &str,
        f: CommandExecutor<TStorage, TModel>,
    ) -> Self
    where
        T: Command<'a, TModel> + 'static,
    {
        let id = TypeId::of::<T>();
        println!("Adding: {}", name);

        self.commands.insert(name.to_string(), f);
        self.command_names_by_id.insert(id, name.to_string());
        self
    }

    pub fn build(mut self) -> Engine<TModel, TStorage> {
        self.storage.restore(&mut self.model, &self.commands);

        Engine {
            model: Arc::new(RwLock::new(self.model)),
            storage: Arc::new(Mutex::new(self.storage)),
            commands: Arc::new(self.command_names_by_id),
        }
    }
}
