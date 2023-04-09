use crate::models::{EcomModel, Order};
use bincode::{Decode, Encode};
use origo::Command;
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Serialize, Deserialize)]
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
