mod engine;
pub mod storage;
pub use engine::*;

#[macro_export]
macro_rules! origo_engine {
    ($model:ty, $storage:expr, $($y:ty,)+) => {{
        let mut engine = $crate::EngineBuilder::new(<$model>::default(), $storage);
        $crate::origo_engine! {
            engine $model, $($y),+
        }
        engine.build()
    }};

    ($engine:ident $model:ty, $command:ty) => {
        $engine = $engine.register_command::<$command>(stringify!($command), Box::new(|storage, data, model| {
            $crate::storage::Storage::restore_command::<$model, $command>(storage, data, model);
        }));
    };

    ($engine:ident $model:ty, $command:ty, $($commands:ty),+) => {
        $crate::origo_engine!{
            $engine $model, $command
        }
        $crate::origo_engine!{
            $engine $model, $($commands),+
        }
    };
}
