use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct EcomModel {
    pub orders: FxHashMap<usize, Order>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Order {
    pub order_id: usize,
    pub name: String,
    pub transport_id: usize,
}
