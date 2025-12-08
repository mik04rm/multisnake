mod shared;

use axum::{
    Router,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use bitvec::prelude::*;
use clap::Parser;
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{self, UnboundedSender};
use uuid::Uuid;

use crate::shared::{GRID_H, GRID_W, Pos, SnakeMessage};

// Helper inline to compute linear index
#[inline]
fn idx(p: &Pos) -> usize {
    p.y as usize * GRID_W as usize + p.x as usize
}

#[derive(Parser)]
struct Args {
    #[arg(default_value = "127.0.0.1:8080")]
    addr: String,
}

struct Client {
    tx: UnboundedSender<Message>,
    snake: VecDeque<Pos>,
}

struct GameState {
    clients: HashMap<Uuid, Client>,
    occupied: BitVec,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let clients: HashMap<Uuid, Client> = HashMap::new();
    let occupied: BitVec = bitvec![0; (GRID_W * GRID_H) as usize];
    let game_state = GameState { clients, occupied };
    let app = Router::new()
        .route("/", get(ws_handler))
        .with_state(Arc::new(Mutex::new(game_state)));

    let listener = tokio::net::TcpListener::bind(&args.addr).await.unwrap();
    println!("Server running on ws://{}", args.addr);
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(game_state): State<Arc<Mutex<GameState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, game_state))
}

// TODO food spawning and eating
// TODO better messaging
// TODO Isn't lock held for too long?

async fn handle_socket(socket: WebSocket, game_state: Arc<Mutex<GameState>>) {
    let client_id = Uuid::new_v4();

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Each client gets a channel to receive broadcast messages
    let (tx, mut rx) = mpsc::unbounded_channel();

    let midx = GRID_W / 2;
    let midy = GRID_H / 2;

    let initial_snake = VecDeque::from([
        Pos { x: midx, y: midy },
        Pos {
            x: midx - 1,
            y: midy,
        },
        Pos {
            x: midx - 2,
            y: midy,
        },
    ]);

    game_state.lock().unwrap().clients.insert(
        client_id,
        Client {
            tx: tx.clone(),
            snake: initial_snake.clone(),
        },
    );
    // From now on, the new client will receive messages from other clients

    // TODO there can be instant collisons
    // Potentially invalid states.
    for p in initial_snake.iter() {
        game_state.lock().unwrap().occupied.set(idx(p), true);
    }

    let msg = SnakeMessage::InitSnake {
        client_id,
        segments: initial_snake,
    };

    for client in game_state.lock().unwrap().clients.values() {
        let json = serde_json::to_string(&msg).unwrap();
        let _ = client.tx.send(Message::Text(json.into()));
        println!("Sent InitSnake (about {:?}) to client", client_id);
    }

    // Task to forward messages from other clients to this WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Read messages from this client, process them, and broadcast to others
    while let Some(Ok(msg)) = ws_rx.next().await {
        // Assert that snakes are consistent with occupied grid
        let mut game_state_guard = game_state.lock().unwrap();
        let GameState { clients, occupied } = &mut *game_state_guard;
        // let clients_guard = game_state.lock().unwrap().clients;
        for client in clients.values() {
            for p in client.snake.iter() {
                if !occupied[idx(p)] {
                    println!(
                        "Inconsistency detected for client {:?} at position {:?}",
                        client.tx, p
                    );
                }
            }
        }
        // Assert that not occupied cells are indeed free
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let p = Pos { x, y };
                if !clients.values().any(|c| c.snake.iter().any(|&s| s == p)) && occupied[idx(&p)] {
                    println!("Inconsistency detected at position {:?}", p);
                }
            }
        }

        let txt = match msg {
            Message::Text(t) => t,
            _ => break,
        };

        if let Ok(SnakeMessage::Move { dx, dy, .. }) = serde_json::from_str::<SnakeMessage>(&txt) {
            println!("Client {:?} moved by ({}, {})", client_id, dx, dy);
            // Update the snake position

            let client = clients.get_mut(&client_id).unwrap();

            let head = client.snake.front().copied().unwrap();
            let tail = client.snake.back().copied().unwrap();
            let new_head = Pos {
                x: (head.x + dx + GRID_W) % GRID_W,
                y: (head.y + dy + GRID_H) % GRID_H,
            };

            if occupied[idx(&new_head)] {
                // Collision detected
                println!("Collision detected for client {:?}", client_id);
                for p in client.snake.iter() {
                    occupied.set(idx(p), false);
                }

                let dead_msg = SnakeMessage::Dead { client_id };
                let dead_json = serde_json::to_string(&dead_msg).unwrap();
                for client in clients.values() {
                    let _ = client.tx.send(Message::Text(dead_json.clone().into()));
                }
                break;
            } else {
                client.snake.push_front(new_head);
                client.snake.pop_back();
                // println!("Client {:?} moved to {:?}", moved_client_id, new_head);
                occupied.set(idx(&new_head), true);
                occupied.set(idx(&tail), false);
            }
        }

        // Broadcast to all other clients
        for (other_id, other) in clients.iter() {
            if *other_id == client_id {
                continue;
            }
            let _ = other.tx.send(Message::Text(txt.clone()));
        }
    }
    let _ = tx.send(Message::Close(None));
    // Remove the client when disconnected.
    game_state.lock().unwrap().clients.remove(&client_id);
}
