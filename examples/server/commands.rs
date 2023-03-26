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

#[derive(Serialize, Deserialize)]
pub struct InsertOrder2<'a> {
    pub order_id: usize,
    pub name: &'a str,
    pub transport_id: usize,
}

impl<'a> Command<'a, EcomModel> for InsertOrder2<'a> {
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
        "InsertOrder2"
    }
}
