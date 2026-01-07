use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio::sync::{broadcast::error::RecvError, mpsc};
use uuid::Uuid;

use crate::room_manager::RoomManager;
use multisnake_shared::{LobbyUpdate, SnakeMessage};

pub struct RoomContext {
    pub room_manager: Arc<Mutex<RoomManager>>,
    pub lobby_tx: broadcast::Sender<LobbyUpdate>,
    pub room_id: u32,
}

pub struct TuiContext {
    pub lobby_tx: broadcast::Sender<LobbyUpdate>,
    pub snapshot_req_tx: mpsc::UnboundedSender<mpsc::UnboundedSender<Vec<LobbyUpdate>>>,
}

pub async fn in_room_handler(
    ws: WebSocketUpgrade,
    State(ctx): State<Arc<RoomContext>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_in_room_connection(socket, ctx))
}

pub async fn in_tui_handler(
    ws: WebSocketUpgrade,
    State(tui_ctx): State<Arc<TuiContext>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_in_tui_connection(socket, tui_ctx))
}

async fn handle_in_room_connection(socket: WebSocket, ctx: Arc<RoomContext>) {
    let client_id = Uuid::new_v4();
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    {
        let mut room_guard = ctx.room_manager.lock().await;
        room_guard.add_client(client_id, tx.clone());

        let init_msg = room_guard.new_init_message(client_id);
        if let Ok(json) = serde_json::to_string(&init_msg) {
            let _ = tx.send(Message::Text(json.into()));
        }

        let player_count = room_guard.clients.len();
        let _ = ctx.lobby_tx.send(LobbyUpdate {
            room_id: ctx.room_id,
            player_count,
        });
    }

    loop {
        tokio::select! {
            // Outbound: From Room manager -> WebSocket
            Some(msg) = rx.recv() => {
                if ws_tx.send(msg).await.is_err() { break; }
            }

            // Inbound: From WebSocket -> Room manager
            result = ws_rx.next() => {
                match result {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(SnakeMessage::MoveIntent { dx, dy }) = serde_json::from_str(&text) {
                            let mut room_guard = ctx.room_manager.lock().await;
                            room_guard.queue_move(&client_id, dx, dy);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    let _ = ws_tx.close().await;

    let mut room_guard = ctx.room_manager.lock().await;
    room_guard.remove_client(&client_id);
    let player_count = room_guard.clients.len();
    let _ = ctx.lobby_tx.send(LobbyUpdate {
        room_id: ctx.room_id,
        player_count,
    });

    println!(
        "Client {:?} disconnected and removed from state.",
        client_id
    );
}

async fn handle_in_tui_connection(mut socket: WebSocket, tui_ctx: Arc<TuiContext>) {
    let mut rx = tui_ctx.lobby_tx.subscribe();
    let (response_tx, mut response_rx) = mpsc::unbounded_channel();
    tui_ctx.snapshot_req_tx.send(response_tx).unwrap();
    let initial_snapshot = response_rx.recv().await.unwrap();

    for update in initial_snapshot {
        let json = serde_json::to_string(&update).unwrap();
        if socket.send(Message::Text(json.into())).await.is_err() {
            return; // TUI disconnected
        }
    }

    // Listen for broadcasted updates and forward them to the TUI websocket
    loop {
        match rx.recv().await {
            Ok(update) => {
                let json = serde_json::to_string(&update).unwrap();
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break; // TUI disconnected
                }
            }
            Err(RecvError::Lagged(_)) => continue,
            Err(RecvError::Closed) => break,
        }
    }
}
