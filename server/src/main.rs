mod commands;
mod models;
use std::time::Instant;

use origo::storage::DiskStorage;
use tide::{Body, Request};
use {commands::*, models::*};

/// Makes it easier, `req: Request<Db>` in functions
/// instead of `req: Request<Engine<EComModel>>`
type Db = origo::Engine<EcomModel, DiskStorage>;

/// We should take a snapshot after this amount of commited commands
const SNAPSHOT_COMMAND_COUNT: u64 = 100;

#[async_std::main]
async fn main() -> tide::Result<()> {
    env_logger::init();
    let instant = Instant::now();

    let db = origo::origo_engine! {
        EcomModel,
        DiskStorage::new("./data/test.origors"),
        InsertOrder,
    };

    db.snapshot_command_count(SNAPSHOT_COMMAND_COUNT);

    log::info!("Startup: {}ms", instant.elapsed().as_millis());

    let order_count = db.query(|m| m.orders.len());
    log::info!("{order_count} orders in db");

    if db.query(|m| m.orders.is_empty()) {
        for i in 0..120 {
            insert_test_data(&db, &i);
        }

        log::info!("Inserted test-data");
    }

    let mut app = tide::with_state(db);
    app.at("/orders")
        .post(place_order)
        .at("/:id")
        .get(fetch_order);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn insert_test_data(db: &Db, count: &i32) {
    let test_data = InsertOrder {
        order_id: *count as usize,
        name: String::from("TestOrder"),
        transport_id: 2,
    };
    _ = db.execute(test_data);
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

async fn place_order<'a>(mut req: Request<Db>) -> tide::Result {
    match req.body_json::<InsertOrder>().await {
        Ok(command) => {
            _ = req.state().execute(command);
            Ok(tide::Response::new(200))
        }
        Err(e) => Err(e),
    }
}
