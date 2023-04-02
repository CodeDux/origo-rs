use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, collections::HashMap, sync::Arc};

use crate::storage::Storage;

pub type CommandRestoreFn<TStorage, TModel> = Box<dyn Fn(&TStorage, &[u8], &mut TModel)>;

pub trait Command<'de, TModel>: Serialize + Deserialize<'de> {
    fn execute(&self, model: &mut TModel);
}

pub struct Engine<TModel, TStorage> {
    model: Arc<RwLock<TModel>>,
    storage: Arc<Mutex<TStorage>>,
    typeid_names: Arc<HashMap<TypeId, String>>,
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
        let name: &str = self
            .typeid_names
            .get(&TypeId::of::<T>())
            .expect("Couldn't find command_name");

        // We lock storage before the model so we can allow queries during the possible storage IO
        // This is the reason for storing `storage` and `model` in separate locks
        let mut storage = self.storage.lock();
        storage.prepare_command::<TModel, T>(name, command);

        // Here we lock the model so no queries can happen before the new state is applied and
        // commited.
        //
        // We also keep holding the lock on storage since we don't know if we are "safe" until
        // we commited/flushed the command
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
            typeid_names: self.typeid_names.clone(),
        }
    }
}

/// Used to build and restore an engine for `TModel` with `TStorage`
///
/// **DON'T USE THIS, use the [`crate::origo_engine`] macro**
pub struct EngineBuilder<TModel, TStorage> {
    model: TModel,
    storage: TStorage,
    restore_fns: HashMap<String, CommandRestoreFn<TStorage, TModel>>,
    typeid_names: HashMap<TypeId, String>,
}

impl<TModel, TStorage: Storage> EngineBuilder<TModel, TStorage> {
    pub fn new(model: TModel, storage: TStorage) -> EngineBuilder<TModel, TStorage> {
        EngineBuilder {
            model,
            storage,
            restore_fns: HashMap::new(),
            typeid_names: HashMap::new(),
        }
    }

    /// Register command that the engine should be able to execute AND store + restore
    ///
    /// It works by taking the `TypeId` of `T` while also receiving the name of the command,
    /// with this it creates a runtime mapping that is used to map stored data with runtime types
    ///
    /// The `TypeId` and name is used by the engine to map between the Command -> TypeId -> Name when:
    /// - Executing a command, we get the `TypeId` of the executing command and with that we get the name,
    /// the name is then stored with the serialized data.
    /// - Restoring the model, we have the names stored with the serialized data and for example:
    /// The [`crate::JsonStorage`] uses that name to fetch the [`CommandRestoreFn`],
    /// then it knows how to deserialize that command from the journal
    pub fn register_command<'a, T: Command<'a, TModel> + 'static>(
        mut self,
        name: &str,
        restore_fn: CommandRestoreFn<TStorage, TModel>,
    ) -> Self {
        println!("Adding: {}", name);

        self.restore_fns.insert(name.to_string(), restore_fn);
        self.typeid_names
            .insert(TypeId::of::<T>(), name.to_string());
        self
    }

    pub fn build(mut self) -> Engine<TModel, TStorage> {
        self.storage.restore(&mut self.model, &self.restore_fns);

        Engine {
            model: Arc::new(RwLock::new(self.model)),
            storage: Arc::new(Mutex::new(self.storage)),
            typeid_names: Arc::new(self.typeid_names),
        }
    }
}
