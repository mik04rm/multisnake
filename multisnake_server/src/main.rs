mod room_manager;
mod socket_handlers;

use axum::{Router, routing::get};
use clap::Parser;
use room_manager::RoomManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time;

use multisnake_shared::LobbyUpdate;
use multisnake_shared::N_ROOMS;

use crate::socket_handlers::RoomContext;
use crate::socket_handlers::TuiContext;

const BROADCAST_CAPACITY: usize = 1024;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "127.0.0.1:4040")]
    addr: String,
    #[arg(default_value = "100")]
    tick_duration_ms: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (lobby_tx, _) = broadcast::channel::<LobbyUpdate>(BROADCAST_CAPACITY);
    let (snapshot_req_tx, mut snapshot_req_rx) =
        mpsc::unbounded_channel::<mpsc::UnboundedSender<Vec<LobbyUpdate>>>();

    let mut app = Router::new();

    let mut managers = Vec::new();

    for i in 1..=N_ROOMS {
        let room_manager = Arc::new(Mutex::new(RoomManager::new(args.tick_duration_ms)));

        managers.push((i, room_manager.clone()));

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

        app = app.route(&path, get(socket_handlers::in_room_handler).with_state(ctx));

        println!("Registered room at ws://{}{}", args.addr, path);
    }

    tokio::spawn(async move {
        while let Some(reply_tx) = snapshot_req_rx.recv().await {
            let mut snapshot = Vec::new();

            for (id, manager) in &managers {
                let count = {
                    let guard = manager.lock().await;
                    guard.clients.len()
                };
                snapshot.push(LobbyUpdate {
                    room_id: *id,
                    player_count: count,
                });
            }

            let _ = reply_tx.send(snapshot);
        }
    });

    let tui_ctx = Arc::new(TuiContext {
        lobby_tx,
        snapshot_req_tx,
    });
    app = app.route(
        "/room",
        get(socket_handlers::in_tui_handler).with_state(tui_ctx),
    );

    let listener = TcpListener::bind(&args.addr).await.unwrap();
    println!("Server running on ws://{}", args.addr);
    axum::serve(listener, app).await.unwrap();
}
