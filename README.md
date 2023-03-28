# **THIS IS JUST A PROOF-OF-CONCEPT PROJECT FOR ME/MYSELF&I**
*Nothing special, just trying things with Rust to learn more about it.*

This is a in-memory database that journals commands to disk and replays the commands on startup to recreate the state.

See the [Example](examples/server/) for example of implementation in a http-server (tide)

## Run example
`cargo run -r --example server`

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
pub struct InsertOrder<'a> {
    pub order_id: usize,
    pub name: &'a str,
    pub transport_id: usize,
}

impl<'a> Command<'a, EcomModel> for InsertOrder<'a> {
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

    fn identifier() -> &'static str {
        "InsertOrder"
    }
}
```

### Create engine for model and commands
This is done with the `origo_engine!` macro
```
origo_engine! {
    MainModelType: "path to store the journal",
    CommandTypeToSupport,
    CommandTypeToSupport,
    ... more CommandTypesToSupport
};
```
like this
```rust
use origo::origo_engine;
//..
let engine = origo_engine! {
    EcomModel: "./data/test.origors",
    InsertOrder,
    InsertOrder2,
};
```

### Use the engine
#### Query
```rust
let ids = [12, 24, 2285];
let orders: Vec<Order> = engine.query(|model| {
    ids.iter()
        .map(|id| model.orders.get(&id))
        .filter(|o| o.is_some())
        .map(|o| o.unwrap().clone())
        .collect()
});
```
#### Execute Commands
```rust
engine.execute(&InsertOrder {
    name: &fake_name(),
    order_id: fake_id(),
    transport_id: fake_id(),
});
```

## Threading
The engine implements `Clone` and one instance should be created on startup, then pass clones to threads that needs to execute commands or query data.

The model is wrapped in a `RwLock` to support multiple queries or one command at any given time.
```rust
let en = engine.clone();
let handle = thread::spawn(move || {
    en.execute(&InsertOrder {
        name: &fake_name(),
        order_id: fake_id(),
        transport_id: fake_id(),
    });
});
```