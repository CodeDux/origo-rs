use bincode::{Decode, Encode};
use parking_lot::{Mutex, RwLock};
use std::{
    any::TypeId,
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};

use crate::storage::Storage;

pub type CommandRestoreFn<TModel> =
    Box<dyn Fn(&[u8], &mut TModel, bincode::config::Configuration) -> bool>;

pub trait Command<TModel>: Encode + Decode {
    fn execute(&self, model: &mut TModel);
}

pub struct Engine<TModel, TStorage> {
    model: Arc<RwLock<TModel>>,
    storage: Arc<Mutex<TStorage>>,
    typeid_names: Arc<HashMap<TypeId, String>>,
    snapshot_command_count: Arc<AtomicU64>,
}

impl<TModel: Encode + Decode + Send + Sync + 'static, TStorage: Storage + Send + 'static>
    Engine<TModel, TStorage>
{
    /// How many commands are allowed before (automatically) taking a snapshot
    pub fn snapshot_command_count(&self, count: u64) {
        self.snapshot_command_count
            .store(count, std::sync::atomic::Ordering::Relaxed);
    }

    /// Execute the given command against the current model
    ///
    /// Commands execute in exclusive mode,
    /// meaning that no other writes OR queries will happen until the command finishes
    /// (The model is ReadWriteLocked)
    ///
    /// Before executing the command it's written to the journal
    pub fn execute<T>(&self, command: T)
    where
        T: Command<TModel> + 'static,
    {
        let name: &str = self
            .typeid_names
            .get(&TypeId::of::<T>())
            .expect("Couldn't find command name");

        // We lock storage before the model so we can allow queries during the possible storage IO
        // This is the reason for storing `storage` and `model` in separate locks
        let mut storage = self.storage.lock();

        storage.prepare(name, &command);

        // Here we lock the model so no queries can happen before the new state is applied
        // and committed.
        let mut model = self.model.write();
        command.execute(&mut model);
        let command_count = storage.commit();

        // Since we still hold the lock on storage (and no writes can happen until we release it)
        // we check if we should take a snapshot
        if command_count
            == self
                .snapshot_command_count
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            let clone = self.clone();
            std::thread::spawn(move || {
                let mut storage2 = clone.storage.lock();
                let model2 = clone.model.read();
                storage2.snapshot(&*model2);
            });
        }
    }

    /// Execute the given query against the current model
    ///
    /// Multiple queries can execute against the model at the same time
    /// but no writes will happen during queries (The model is ReadWriteLocked)
    #[inline(always)]
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
            snapshot_command_count: self.snapshot_command_count.clone(),
        }
    }
}

/// Used to build and restore an engine for `TModel` with `TStorage`
///
/// **DON'T USE THIS, use the [`crate::origo_engine`] macro**
pub struct EngineBuilder<TModel, TStorage> {
    model: TModel,
    storage: TStorage,
    restore_fns: HashMap<String, CommandRestoreFn<TModel>>,
    typeid_names: HashMap<TypeId, String>,
}

impl<TModel: Default + Decode, TStorage: Storage> EngineBuilder<TModel, TStorage> {
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
    /// The [`crate::storage::DiskStorage`] uses that name to fetch the [`CommandRestoreFn`],
    /// then it knows how to deserialize that command from the journal
    pub fn register_command<T: Command<TModel> + 'static>(
        mut self,
        persistent_identifier: &str,
    ) -> Self {
        log::debug!("Registering command: {}", persistent_identifier);

        let restore_fn: CommandRestoreFn<TModel> = Box::new(|data, model, config| {
            match bincode::decode_from_slice::<T, _>(data, config) {
                Ok((command, _)) => {
                    command.execute(model);
                    true
                }
                Err(_) => false, // TODO: report the error to caller in some way
            }
        });

        match self
            .restore_fns
            .insert(persistent_identifier.to_string(), restore_fn)
        {
            Some(_) => panic!(
                "Command with name {} already registered",
                persistent_identifier
            ),
            None => self
                .typeid_names
                .insert(TypeId::of::<T>(), persistent_identifier.to_string()),
        };

        self
    }

    pub fn build(mut self) -> Engine<TModel, TStorage> {
        self.model = self.storage.restore(&self.restore_fns);

        Engine {
            model: Arc::new(RwLock::new(self.model)),
            storage: Arc::new(Mutex::new(self.storage)),
            typeid_names: Arc::new(self.typeid_names),
            snapshot_command_count: Arc::new(AtomicU64::new(u64::MAX)),
        }
    }
}
