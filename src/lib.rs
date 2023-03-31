mod engine;
mod storage;
pub use engine::*;
pub use storage::*;

#[macro_export]
macro_rules! origo_engine {
    ($model:ty, $storage:expr, $($y:ty,)+) => {{
        let mut engine = $crate::EngineBuilder::new(<$model>::default(), $storage);
        $crate::origo_engine! {
            engine $model, $($y),+
        }
        engine.build()
    }};

    ($engine:ident $model:ty, $e:ty) => {
        $engine = $engine.register_command::<$e>(stringify!($e), Box::new(|storage, data, model| {
            $crate::Storage::deserialize::<$model, $e>(storage, data, model);
        }));
    };

    ($engine:ident $model:ty, $e:ty, $($y:ty),+) => {
        $engine = $engine.register_command::<$e>(stringify!($e), Box::new(|storage, data, model| {
            $crate::Storage::deserialize::<$model, $e>(storage, data, model);
        }));
        $crate::origo_engine!{
            $engine $model, $($y),+
        }
    };
}
