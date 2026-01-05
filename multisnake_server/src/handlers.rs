use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::state::GameState;
use multisnake_shared::{LobbyUpdate, SnakeMessage};

pub struct RoomContext {
    pub game_state: Arc<Mutex<GameState>>,
    pub lobby_tx: tokio::sync::broadcast::Sender<LobbyUpdate>,
    pub room_id: u32,
}

pub(crate) async fn in_game_handler(
    ws: WebSocketUpgrade,
    State(ctx): State<Arc<RoomContext>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_in_game_connection(socket, ctx))
}

pub(crate) async fn in_tui_handler(
    ws: WebSocketUpgrade,
    State(lobby_tx): State<tokio::sync::broadcast::Sender<LobbyUpdate>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_in_tui_connection(socket, lobby_tx))
}

async fn handle_in_game_connection(socket: WebSocket, ctx: Arc<RoomContext>) {
    let client_id = Uuid::new_v4();
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    {
        let mut gs = ctx.game_state.lock().await;
        gs.add_client(client_id, tx.clone());

        let init_msg = gs.new_init_message(client_id);
        if let Ok(json) = serde_json::to_string(&init_msg) {
            let _ = tx.send(Message::Text(json.into()));
        }

        let player_count = gs.clients.len();
        let _ = ctx.lobby_tx.send(LobbyUpdate {
            room_id: ctx.room_id,
            player_count,
        });
    }

    loop {
        tokio::select! {
            // Outbound: From Game State -> WebSocket
            Some(msg) = rx.recv() => {
                if ws_tx.send(msg).await.is_err() { break; }
            }

            // Inbound: From WebSocket -> Game State
            result = ws_rx.next() => {
                match result {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(SnakeMessage::MoveIntent { dx, dy }) = serde_json::from_str(&text) {
                            let mut gs = ctx.game_state.lock().await;
                            gs.queue_move(&client_id, dx, dy);

                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    let _ = ws_tx.close().await;

    let mut gs = ctx.game_state.lock().await;
    gs.remove_client(&client_id);
    let player_count = gs.clients.len();
    let _ = ctx.lobby_tx.send(LobbyUpdate {
        room_id: ctx.room_id,
        player_count,
    });

    println!(
        "Client {:?} disconnected and removed from state.",
        client_id
    );
}

async fn handle_in_tui_connection(
    mut socket: WebSocket,
    lobby_tx: tokio::sync::broadcast::Sender<LobbyUpdate>,
) {
    let mut rx = lobby_tx.subscribe();

    // Listen for broadcasted updates and push them to the TUI websocket
    while let Ok(update) = rx.recv().await {
        let json = serde_json::to_string(&update).unwrap();
        if socket.send(Message::Text(json.into())).await.is_err() {
            break; // TUI closed or disconnected
        }
    }
}
