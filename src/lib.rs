mod engine;
mod storage;
pub use engine::*;
pub use storage::*;

#[macro_export]
macro_rules! origo_engine {
    ($model:ty:$path:literal, $($y:ty,)+) => {{
        let mut engine = $crate::EngineBuilder::<$model>::new($path);
        $crate::__attach_command! {
            engine $($y);+;
        }
        engine.build()
    }};
}

/// Don't use this, it's internal
#[macro_export]
macro_rules! __attach_command {
    ($engine:ident $e:ty;) => {
        $engine = $engine.register_command::<$e>(Box::new(|data, model| {
            let command = serde_json::from_slice::<$e>(data)
                .unwrap();
            $crate::Command::execute(&command, model);
        }));

    };

    ($engine:ident $e:ty;$($y:ty;);+) => {
        $engine = $engine.register_command::<$e>(Box::new(|data, model| {
            let command = serde_json::from_slice::<$e>(data)
                .unwrap();
            $crate::Command::execute(&command, model);
        }));
        $crate::__attach_command!{
            $engine $($y)+;
        }
    };
}
