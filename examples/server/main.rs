mod commands;
mod models;
use origo::{origo_engine, Engine};
use tide::Body;
use tide::Request;
use {commands::*, models::*};

#[async_std::main]
async fn main() -> tide::Result<()> {
    let engine = origo_engine! {
        EcomModel: "./data/test.origors",
        InsertOrder,
    };

    insert_test_data(&engine);

    let mut app = tide::with_state(engine);
    app.at("/orders")
        .post(place_order)
        .at("/:id")
        .get(fetch_order);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn insert_test_data(engine: &Engine<EcomModel>) -> () {
    if engine.query(|m| m.orders.len() == 0) {
        let test_data = InsertOrder {
            order_id: 1,
            name: "TestOrder".to_string(),
            transport_id: 2,
        };
        engine.execute(&test_data);
        println!(
            "Inserted test-data: {}",
            serde_json::to_string_pretty(&test_data).unwrap()
        )
    }
}

async fn fetch_order(req: Request<Engine<EcomModel>>) -> tide::Result {
    let id = req.param("id").unwrap().parse::<usize>().unwrap();
    let result = req.state().query(|m| match m.orders.get(&id) {
        Some(order) => Some(order.clone()),
        None => None,
    });

    Ok(match result {
        Some(order) => {
            let mut res = tide::Response::new(200);
            res.set_body(Body::from_json(&order).unwrap());
            res
        }
        None => tide::Response::new(404),
    })
}

async fn place_order(mut req: Request<Engine<EcomModel>>) -> tide::Result {
    match req.body_json::<InsertOrder>().await {
        Ok(command) => {
            req.state().execute(&command);
            Ok(tide::Response::new(200))
        }
        Err(e) => Err(e),
    }
}
