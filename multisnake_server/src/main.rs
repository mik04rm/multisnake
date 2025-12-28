mod handlers;
mod state;

use axum::{Router, routing::get};
use clap::Parser;
use state::GameState;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
struct Args {
    #[arg(default_value = "127.0.0.1:8080")]
    addr: String,
}

const TICK_INTERVAL_MS: u64 = 200;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut app = Router::new();

    for i in 1..=3 {
        let room_state = Arc::new(Mutex::new(GameState::new()));


        let room_state_clone = room_state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(TICK_INTERVAL_MS));
            loop {
                interval.tick().await;
                let mut gs = room_state_clone.lock().unwrap();
                gs.tick();
            }
        });

        let path = format!("/room/{}", i);

        app = app.route(
            &path, 
            get(handlers::ws_handler).with_state(room_state)
        );
        
        println!("Registered room at ws://{}{}", args.addr, path);
    }

    let listener = tokio::net::TcpListener::bind(&args.addr).await.unwrap();
    println!("Server running on ws://{}", args.addr);
    axum::serve(listener, app).await.unwrap();
}
