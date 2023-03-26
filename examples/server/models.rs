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
