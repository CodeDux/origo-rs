mod commands;
mod models;
use commands::*;
use fake::{faker, Fake};
use models::*;
use origo::origo_engine;

use std::{
    io::stdout,
    thread::{self, JoinHandle},
    time::Instant,
};

fn main() {
    let now = Instant::now();

    let engine = origo_engine! {
        EcomModel: "./data/test.origors",
        InsertOrder,
        InsertOrder2,
    };

    println!("Startup: {}ms", now.elapsed().as_millis());

    // Query orders
    let now = Instant::now();
    let ids = [12, 24, 2285];
    let orders: Vec<Order> = engine.query(|model| {
        ids.iter()
            .map(|id| model.orders.get(&id))
            .filter(|o| o.is_some())
            .map(|o| o.unwrap().clone())
            .collect()
    });
    println!("Query: {}ns", now.elapsed().as_nanos());

    // Print result of query
    for order in orders {
        _ = serde_json::to_writer_pretty(stdout(), &order);
        println!();
    }

    println!();

    let result = engine.query(|model| model.orders.len());
    println!("Total orders before insert: {result}");

    insert_orders(&engine, result);

    let result = engine.query(|model| model.orders.len());
    println!("Total orders after insert: {result}");
}

fn insert_orders(engine: &origo::Engine<EcomModel>, start: usize) {
    let mut join_handles = Vec::<JoinHandle<()>>::with_capacity(10);

    for i in start..start + 1000 {
        let en = engine.clone();
        let handle = thread::spawn(move || {
            en.execute(&InsertOrder {
                name: &fake_name(),
                order_id: fake_id(),
                transport_id: fake_id(),
            });

            en.execute(&InsertOrder2 {
                name: &fake_name(),
                order_id: fake_id(),
                transport_id: fake_id(),
            });
        });
        join_handles.push(handle);

        if i % 100 == 0 {
            for handle in join_handles {
                handle.join().unwrap();
            }
            join_handles = Vec::<JoinHandle<()>>::with_capacity(10);
        }
    }

    for handle in join_handles {
        handle.join().unwrap();
    }
}

fn fake_name() -> String {
    faker::name::en::Name().fake()
}

fn fake_id() -> usize {
    (1337..1_000_000).fake::<usize>()
}
