mod handlers;
mod room_manager;

use axum::{Router, routing::get};
use clap::Parser;
use room_manager::RoomManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::time;

use multisnake_shared::LobbyUpdate;
use multisnake_shared::N_ROOMS;

use crate::handlers::RoomContext;

const BROADCAST_CAPACITY: usize = 1024;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "127.0.0.1:8080")]
    addr: String,
    #[arg(default_value = "100")]
    tick_duration_ms: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (lobby_tx, _) = broadcast::channel::<LobbyUpdate>(BROADCAST_CAPACITY);

    let mut app = Router::new();

    for i in 1..=N_ROOMS {
        let room_manager = Arc::new(Mutex::new(RoomManager::new(args.tick_duration_ms)));

        let ctx = Arc::new(RoomContext {
            room_manager: room_manager.clone(),
            lobby_tx: lobby_tx.clone(),
            room_id: i,
        });

        let room_manager_clone = room_manager.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(args.tick_duration_ms as u64));
            loop {
                interval.tick().await;
                let mut room_guard = room_manager_clone.lock().await;
                room_guard.tick();
            }
        });

        let path = format!("/room/{}", i);

        app = app.route(&path, get(handlers::in_room_handler).with_state(ctx));

        println!("Registered room at ws://{}{}", args.addr, path);
    }

    app = app.route("/room", get(handlers::in_tui_handler).with_state(lobby_tx));

    let listener = TcpListener::bind(&args.addr).await.unwrap();
    println!("Server running on ws://{}", args.addr);
    axum::serve(listener, app).await.unwrap();
}
