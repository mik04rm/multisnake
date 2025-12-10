use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, stream::StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::state::{GameState, MoveResult};
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

    // Channel for this client handler to communicate with other clients handlers
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Add client
    let initial_snake = {
        let mut game_state_guard = game_state.lock().unwrap();
        game_state_guard.add_client(client_id, tx.clone())
    };

    let msg = SnakeMessage::InitSnake {
        client_id,
        segments: initial_snake,
    };

    // Broadcast InitSnake to all clients
    {
        let game_state_guard = game_state.lock().unwrap();
        let json = serde_json::to_string(&msg).unwrap();
        for client in game_state_guard.clients.values() {
            let _ = client.tx.send(Message::Text(json.clone().into()));
        }
        println!("New Client {:?} joined", client_id);
    }

    let food_msg = SnakeMessage::FoodPos {
        pos: game_state.lock().unwrap().food,
    };
    let food_json = serde_json::to_string(&food_msg).unwrap();
    tx.send(Message::Text(food_json.into())).unwrap();

    // Forwarder task: internal channel -> WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Receiver loop: this client WebSocket -> game state update and broadcast to other clients
    while let Some(Ok(msg)) = ws_rx.next().await {
        let mut game_state_guard = game_state.lock().unwrap();

        let txt = match msg {
            Message::Text(t) => t,
            _ => break,
        };

        if let Ok(SnakeMessage::Move { dx, dy, .. }) = serde_json::from_str::<SnakeMessage>(&txt) {
            let move_result = game_state_guard.move_snake(&client_id, dx, dy);

            match move_result {
                MoveResult::Moved => {}
                MoveResult::Ate => {
                    println!("Client {:?} ate food!", client_id);
                    let ate_msg = SnakeMessage::AteFood { client_id };
                    let ate_json = serde_json::to_string(&ate_msg).unwrap();

                    // Broadcast food consumption
                    for client in game_state_guard.clients.values() {
                        let _ = client.tx.send(Message::Text(ate_json.clone().into()));
                    }

                    let food_pos = game_state_guard.food;
                    let food_msg = SnakeMessage::FoodPos { pos: food_pos };
                    let food_json = serde_json::to_string(&food_msg).unwrap();

                    // Broadcast new food position
                    for client in game_state_guard.clients.values() {
                        let _ = client.tx.send(Message::Text(food_json.clone().into()));
                    }
                }
                MoveResult::Collision => {
                    let dead_msg = SnakeMessage::Dead { client_id };
                    let dead_json = serde_json::to_string(&dead_msg).unwrap();

                    // Broadcast death
                    for client in game_state_guard.clients.values() {
                        let _ = client.tx.send(Message::Text(dead_json.clone().into()));
                    }

                    break;
                }
            }
        }

        for (other_id, other) in game_state_guard.clients.iter() {
            if *other_id == client_id {
                continue;
            }
            let _ = other.tx.send(Message::Text(txt.clone()));
        }
    }

    let _ = tx.send(Message::Close(None));

    // Cleanup
    game_state.lock().unwrap().remove_client(&client_id);
    println!("Client {:?} disconnected", client_id);
}
