# origo-rs

This is a in-memory database that journals commands to disk and replays the commands on startup to recreate the state, the code can be "broken" at any given day, **THIS IS A PROOF-OF-CONCEPT PROJECT, just trying things with Rust to learn more about it.**

## Run server
Run the following in the repository root
```bash
## uses .cargo/config.toml
cargo run server
```
or
```bash
RUST_LOG="tide=off, debug" cargo run -r -p server
```

## How it works
### Declare your models
```rust
// For Origo
use bincode::{Decode, Encode};
use std::collections::HashMap;

#[derive(Encode, Decode, Default)]
pub struct EcomModel {
    pub orders: HashMap<usize, Order>,
}

#[derive(Encode, Decode, Clone)]
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
// For Origo
use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct InsertOrder {
    pub order_id: usize,
    pub name: String,
    pub transport_id: usize,
}

impl Command<EcomModel> for InsertOrder {
    fn execute(&self, model: &mut EcomModel) {
        model.orders.insert(
            self.order_id,
            Order {
                order_id: self.order_id,
                name: self.name.clone(),
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
let db = origo_engine! {
    EcomModel,
    DiskStorage::new("./data/test.origors"),
    InsertOrder,
    // Here you keep listing all the commands that the engine should support
};
```

### Usage
#### Query
```rust
let ids = [12, 24, 2285];
let orders: Vec<Order> = db.query(|model| {
    ids.iter()
        .filter_map(|id| model.orders.get(id))
        .cloned()
        .collect()
});
```
#### Execute Commands
```rust
db.execute(&InsertOrder {
    name: fake_name(),
    order_id: fake_id(),
    transport_id: fake_id(),
});
```

#### Snapshots
Configure automatic snapshots by calling `snapshot_command_count` with the amount of commands allowed before triggering a snapshot.
```rust
db.snapshot_command_count(SNAPSHOT_COMMAND_COUNT);
```

## Threading
The engine implements `Clone` and one instance should be created on startup, then pass clones to threads that needs to execute commands or query data.

The model and storage access internally is wrapped in a `RwLock` to support multiple reads(queries) or one write(command) at any given time.
```rust
let db2 = db.clone();
let handle = thread::spawn(move || {
    db2.execute(&InsertOrder {
        name: fake_name(),
        order_id: fake_id(),
        transport_id: fake_id(),
    });
});
```