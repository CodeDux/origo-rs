use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Encode, Decode, Default)]
pub struct EcomModel {
    pub orders: HashMap<usize, Order>,
}

#[derive(Encode, Decode, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: usize,
    pub name: String,
    pub transport_id: usize,
}
