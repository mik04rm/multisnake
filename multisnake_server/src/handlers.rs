use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, stream::StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::state::GameState;
use multisnake_shared::SnakeMessage;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(game_state): State<Arc<Mutex<GameState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, game_state))
}

async fn handle_socket(socket: WebSocket, game_state: Arc<Mutex<GameState>>) {
    let client_id = Uuid::new_v4();
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Internal Channel: GameState -> WebSocket Task
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Forwarder task
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Add client & send GameInit message (game state snapshot)
    {
        let mut gs = game_state.lock().unwrap();
        gs.add_client(client_id, tx.clone());

        let init_msg = gs.get_init_message(client_id);
        let _ = tx.send(Message::Text(
            serde_json::to_string(&init_msg).unwrap().into(),
        ));
    }

    // Receive inputs
    while let Some(Ok(msg)) = ws_rx.next().await {
        if let Message::Text(text) = msg {
            if let Ok(SnakeMessage::MoveIntent { dx, dy }) = serde_json::from_str(&text) {
                let mut gs = game_state.lock().unwrap();
                gs.queue_move(&client_id, dx, dy);
            }
        }
    }
    let _ = tx.send(Message::Close(None));

    // Cleanup
    game_state.lock().unwrap().remove_client(&client_id);
    println!("Client {:?} disconnected", client_id);
}
