# origo-rs

This is a in-memory database that journals commands to disk and replays the commands on startup to recreate the state, the code can be "broken" at any given day, **THIS IS A PROOF-OF-CONCEPT PROJECT, just trying things with Rust to learn more about it.**

## Run server
Run the following in the repository root
```bash
RUST_LOG="tide=off, debug" cargo run -r -p server
```

## How it works
### Declare your models
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
pub struct EcomModel {
    pub orders: HashMap<usize, Order>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Order {
    pub order_id: usize,
    pub name: String,
    pub transport_id: usize,
}

```

### Declare commands
```rust
use crate::models::{EcomModel, Order};
use origo::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InsertOrder {
    pub order_id: usize,
    pub name: String,
    pub transport_id: usize,
}

impl<'a> Command<'a, EcomModel> for InsertOrder {
    fn execute(&self, model: &mut EcomModel) {
        model.orders.insert(
            self.order_id,
            Order {
                order_id: self.order_id,
                name: self.name.to_string(),
                transport_id: self.transport_id,
            },
        );
    }
}
```

### Create engine for model, storage and commands
This is done with the `origo_engine!` macro
```
origo_engine! {
    MainModelType,
    StorageImplementation,
    CommandTypeToSupport1,
    CommandTypeToSupport2,
    ... more CommandTypesToSupport
};
```
like this
```rust
use origo::origo_engine;
//..
let engine = origo_engine! {
    EcomModel,
    JsonStorage::new("./data/test.origors"),
    InsertOrder,
    // Here you keep listing all the commands that the engine should support
};
```

### Use the engine
#### Query
```rust
let ids = [12, 24, 2285];
let orders: Vec<Order> = engine.query(|model| {
    ids.iter()
        .filter_map(|id| model.orders.get(id))
        .cloned()
        .collect()
});
```
#### Execute Commands
```rust
engine.execute(&InsertOrder {
    name: fake_name(),
    order_id: fake_id(),
    transport_id: fake_id(),
});
```

## Threading
The engine implements `Clone` and one instance should be created on startup, then pass clones to threads that needs to execute commands or query data.

The model and storage access internally is wrapped in a `RwLock` to support multiple reads(queries) or one write(command) at any given time.
```rust
let en = engine.clone();
let handle = thread::spawn(move || {
    en.execute(&InsertOrder {
        name: fake_name(),
        order_id: fake_id(),
        transport_id: fake_id(),
    });
});
```