use crate::storage::Storage;

pub struct NoopStorage;

impl Storage for NoopStorage {
    fn prepare<TModel, T: crate::Command<TModel>>(&mut self, _command_name: &str, _command: &T) {}

    fn commit(&mut self) -> u64 {
        0u64
    }

    fn snapshot<TModel: bincode::Encode>(&mut self, _model: &TModel) {}

    fn restore<TModel: Default + bincode::Decode>(
        &mut self,
        _restore_fns: &std::collections::HashMap<String, crate::CommandRestoreFn<TModel>>,
    ) -> TModel {
        TModel::default()
    }
}
