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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Game state shared between all client handlers
    let game_state = Arc::new(Mutex::new(GameState::new()));

    let app = Router::new()
        .route("/", get(handlers::ws_handler))
        .with_state(game_state);

    let listener = tokio::net::TcpListener::bind(&args.addr).await.unwrap();
    println!("Server running on ws://{}", args.addr);
    axum::serve(listener, app).await.unwrap();
}
