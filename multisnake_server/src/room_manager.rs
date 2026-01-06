use axum::extract::ws::Message;
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use multisnake_shared::{GRID_H, GRID_W, Pos, SnakeMessage};

const GHOST_TIME_MS: u32 = 8000;
const PADDING: i32 = 15;
const INITIAL_SNAKE_LENGTH: u32 = 5;

pub struct Client {
    pub tx: UnboundedSender<Message>,
    pub snake: VecDeque<Pos>,
    pub dx: i32,
    pub dy: i32,
    pub next_dx: i32,
    pub next_dy: i32,
    pub ghost_ticks: u32,
}

pub struct RoomManager {
    pub clients: HashMap<Uuid, Client>,

    // 2D grid flattened to 1D. Values > 1 indicate collision.
    pub occupied: Vec<u8>,

    pub food: Pos,

    // New players to be added next tick.
    pub pending_joins: HashMap<Uuid, VecDeque<Pos>>,

    pub tick_duration_ms: u32,
}

impl RoomManager {
    pub fn new(tick_duration_ms: u32) -> Self {
        Self {
            clients: HashMap::new(),
            occupied: vec![0; (GRID_W * GRID_H) as usize],
            food: Pos { x: 5, y: 5 },
            pending_joins: HashMap::new(),
            tick_duration_ms,
        }
    }

    pub fn add_client(&mut self, client_id: Uuid, tx: UnboundedSender<Message>) {
        let initial_snake = initial_snake_segments(INITIAL_SNAKE_LENGTH);

        self.clients.insert(
            client_id,
            Client {
                tx,
                snake: initial_snake.clone(),
                dx: 0,
                dy: -1,
                next_dx: 0,
                next_dy: -1,
                ghost_ticks: GHOST_TIME_MS / self.tick_duration_ms + 1,
            },
        );

        // Add to buffer so existing players see them next tick.
        self.pending_joins.insert(client_id, initial_snake);
    }

    pub fn remove_client(&mut self, client_id: &Uuid) {
        if let Some(client) = self.clients.remove(client_id) {
            if client.ghost_ticks == 0 {
                for p in &client.snake {
                    assert!(is_in_bounds(p));
                    self.occupied[idx(p)] -= 1;
                }
            }
        }
    }

    pub fn queue_move(&mut self, client_id: &Uuid, dx: i32, dy: i32) {
        if let Some(client) = self.clients.get_mut(client_id) {
            // Prevent 180 degree turns
            if dx != -client.dx || dy != -client.dy {
                client.next_dx = dx;
                client.next_dy = dy;
            }
        }
    }

    pub fn new_init_message(&self, my_id: Uuid) -> SnakeMessage {
        SnakeMessage::OnJoin {
            my_id,
            snakes: self
                .clients
                .iter()
                .map(|(k, v)| (*k, v.snake.clone()))
                .collect(),
            tick_duration_ms: self.tick_duration_ms,
        }
    }

    /// The Server tick
    pub fn tick(&mut self) {
        let mut moves_to_broadcast = HashMap::new();
        let mut dead_clients = Vec::new();
        let mut eaters = Vec::new();
        let mut client_ghosts = Vec::new();

        // Calculate moves and wall collisions.
        for (id, client) in self.clients.iter_mut() {
            if client.ghost_ticks > 0 {
                client.ghost_ticks -= 1;
                if client.ghost_ticks == 0 {
                    // Mark occupied grid when ghost mode ends.
                    for p in &client.snake {
                        self.occupied[idx(p)] += 1;
                    }
                } else {
                    client_ghosts.push(*id);
                }
            }

            let (dx, dy, current_head) = {
                (
                    client.next_dx,
                    client.next_dy,
                    *client.snake.front().unwrap(),
                )
            };

            let next_x = current_head.x + dx;
            let next_y = current_head.y + dy;
            let new_head = Pos {
                x: next_x,
                y: next_y,
            };

            // Wall check.
            if !is_in_bounds(&new_head) {
                dead_clients.push(*id);
                continue;
            }

            // Apply move logic (grow or move).
            let old_tail = {
                client.dx = dx;
                client.dy = dy;

                let ate = new_head == self.food && client.ghost_ticks == 0;
                let tail = if ate {
                    None
                } else {
                    Some(*client.snake.back().unwrap())
                };

                client.snake.push_front(new_head);
                if !ate {
                    client.snake.pop_back();
                } else {
                    eaters.push(*id);
                }
                tail
            };

            if client.ghost_ticks == 0 {
                // Update `occupied` grid.
                self.occupied[idx(&new_head)] += 1;
                if let Some(old_tail) = old_tail {
                    self.occupied[idx(&old_tail)] -= 1;
                }
            }
        }

        for _ in 0..eaters.len() {
            self.respawn_food();
        }

        // Snake-to-snake collision check.
        for (id, client) in &self.clients {
            if dead_clients.contains(id) {
                continue;
            } // TODO opt?

            let head = client.snake.front().unwrap();

            // > 1 means that there is a collision, we ignore ghost snakes.
            if client.ghost_ticks == 0 && self.occupied[idx(head)] > 1 {
                dead_clients.push(*id);
            } else {
                moves_to_broadcast.insert(*id, (client.dx, client.dy));
            }
        }

        // Broadcast TickUpdate.
        let update = SnakeMessage::TickUpdate {
            moves: moves_to_broadcast,
            food: self.food,
            deaths: dead_clients.clone(),
            eaters,
            new_snakes: self.pending_joins.clone(),
            ghosts: client_ghosts,
        };

        self.pending_joins.clear();

        if let Ok(json) = serde_json::to_string(&update) {
            self.broadcast(json);
        }

        // Remove clients which died.
        for id in &dead_clients {
            self.remove_client(id);
        }
    }

    fn respawn_food(&mut self) {
        self.food = Pos {
            x: rand::random::<u16>() as i32 % (GRID_W - 2 * PADDING) + PADDING,
            y: rand::random::<u16>() as i32 % (GRID_H - 2 * PADDING) + PADDING,
        };
    }

    fn broadcast(&self, txt: String) {
        let msg = Message::Text(txt.into());
        for client in self.clients.values() {
            let _ = client.tx.send(msg.clone());
        }
    }
}

fn initial_snake_segments(length: u32) -> VecDeque<Pos> {
    let start_x = rand::random::<u16>() as i32 % (GRID_W - 2 * PADDING) + PADDING;
    let start_y = rand::random::<u16>() as i32 % (GRID_H - 2 * PADDING) + PADDING;
    let segments = (0..length)
        .map(|i| Pos {
            x: start_x,
            y: start_y + i as i32,
        })
        .collect();
    segments
}

fn idx(p: &Pos) -> usize {
    p.y as usize * GRID_W as usize + p.x as usize
}

fn is_in_bounds(p: &Pos) -> bool {
    p.x >= 0 && p.x < GRID_W && p.y >= 0 && p.y < GRID_H
}
