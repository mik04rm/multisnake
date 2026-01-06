mod handlers;
mod state;

use axum::{Router, routing::get};
use clap::Parser;
use multisnake_shared::LobbyUpdate;
use state::GameState;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::time;

use crate::handlers::RoomContext;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "127.0.0.1:8080")]
    addr: String,
    #[arg(default_value = "300")]
    tick_duration_ms: u64,
}

const N_ROOMS: u32 = 3;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // TODO: minor check 100 bound?
    let (lobby_tx, _) = broadcast::channel::<LobbyUpdate>(100);

    let mut app = Router::new();

    for i in 1..=N_ROOMS {
        let room_state = Arc::new(Mutex::new(GameState::new(args.tick_duration_ms)));

        let ctx = Arc::new(RoomContext {
            game_state: room_state.clone(),
            lobby_tx: lobby_tx.clone(),
            room_id: i,
        });

        let room_state_clone = room_state.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(args.tick_duration_ms));
            loop {
                interval.tick().await;
                let mut gs = room_state_clone.lock().await;
                gs.tick();
            }
        });

        let path = format!("/room/{}", i);

        app = app.route(&path, get(handlers::in_game_handler).with_state(ctx));

        println!("Registered room at ws://{}{}", args.addr, path);
    }

    app = app.route("/room", get(handlers::in_tui_handler).with_state(lobby_tx));

    let listener = TcpListener::bind(&args.addr).await.unwrap();
    println!("Server running on ws://{}", args.addr);
    axum::serve(listener, app).await.unwrap();
}
