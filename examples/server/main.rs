mod commands;
mod models;
use std::time::Instant;

use origo::JsonStorage;
use tide::{Body, Request};
use {commands::*, models::*};

/// Makes it easier, `req: Request<Db>` in functions
/// instead of `req: Request<Engine<EComModel>>`
type Db = origo::Engine<EcomModel, JsonStorage>;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let instant = Instant::now();
    let db = origo::origo_engine! {
        EcomModel,
        JsonStorage::new("./data/test.origors"),
        InsertOrder,
    };
    println!("Startup: {}ms", instant.elapsed().as_millis());

    let order_count = db.query(|m| m.orders.len());
    println!("{order_count} orders in db");

    insert_test_data(&db);

    let mut app = tide::with_state(db);
    app.at("/orders")
        .post(place_order)
        .at("/:id")
        .get(fetch_order);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn insert_test_data(db: &Db) {
    if db.query(|m| m.orders.is_empty()) {
        let test_data = InsertOrder {
            order_id: 1,
            name: "TestOrder".to_string(),
            transport_id: 2,
        };
        db.execute(&test_data);
        println!(
            "Inserted test-data: {}",
            serde_json::to_string_pretty(&test_data).unwrap()
        )
    }
}

async fn fetch_order(req: Request<Db>) -> tide::Result {
    let id = req.param("id").unwrap().parse::<usize>().unwrap();
    let result = req.state().query(|m| m.orders.get(&id).cloned());

    Ok(match result {
        Some(order) => {
            let mut res = tide::Response::new(200);
            res.set_body(Body::from_json(&order).unwrap());
            res
        }
        None => tide::Response::new(404),
    })
}

async fn place_order(mut req: Request<Db>) -> tide::Result {
    match req.body_json::<InsertOrder>().await {
        Ok(command) => {
            req.state().execute(&command);
            Ok(tide::Response::new(200))
        }
        Err(e) => Err(e),
    }
}
