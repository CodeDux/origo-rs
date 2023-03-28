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

    fn identifier() -> &'static str {
        "InsertOrder"
    }
}
