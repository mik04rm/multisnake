use axum::extract::ws::Message;
use bitvec::prelude::*;
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use multisnake_shared::{GRID_H, GRID_W, Pos};

fn idx(p: &Pos) -> usize {
    p.y as usize * GRID_W as usize + p.x as usize
}

pub struct Client {
    pub tx: UnboundedSender<Message>,
    pub snake: VecDeque<Pos>,
}

pub struct GameState {
    pub clients: HashMap<Uuid, Client>,
    pub occupied: BitVec,
    pub food: Pos,
}

pub enum MoveResult {
    Moved,
    Collision,
    Ate,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            occupied: bitvec![0; (GRID_W * GRID_H) as usize],
            food: Pos {
                x: rand::random::<u16>() as i32 % GRID_W,
                y: rand::random::<u16>() as i32 % GRID_H,
            },
        }
    }

    /// Spawns a snake in the middle and marks grid as occupied
    pub fn add_client(&mut self, client_id: Uuid, tx: UnboundedSender<Message>) -> VecDeque<Pos> {
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

        // Mark initial positions as occupied
        // TODO: handle case if some other snake is already there
        for p in initial_snake.iter() {
            self.occupied.set(idx(p), true);
        }

        self.clients.insert(
            client_id,
            Client {
                tx,
                snake: initial_snake.clone(),
            },
        );

        initial_snake
    }

    pub fn move_snake(&mut self, client_id: &Uuid, dx: i32, dy: i32) -> MoveResult {
        let client = self.clients.get(client_id).unwrap();

        let head = client.snake.front().copied().unwrap();
        let tail = client.snake.back().copied().unwrap();

        let new_head = Pos {
            x: (head.x + dx + GRID_W) % GRID_W,
            y: (head.y + dy + GRID_H) % GRID_H,
        };

        // Check collision
        if self.occupied[idx(&new_head)] {
            for p in client.snake.iter() {
                self.occupied.set(idx(p), false);
            }
            return MoveResult::Collision;
        }

        // Check if ate food
        if new_head == self.food {
            // Grow snake by not removing tail
            self.occupied.set(idx(&new_head), true);
            let client = self.clients.get_mut(client_id).unwrap();
            client.snake.push_front(new_head);

            // Spawn new food
            // TODO write it cleaner
            self.food = Pos {
                x: rand::random::<u16>() as i32 % GRID_W,
                y: rand::random::<u16>() as i32 % GRID_H,
            };
            return MoveResult::Ate;
        }

        // Apply Move
        self.occupied.set(idx(&new_head), true);
        self.occupied.set(idx(&tail), false);

        let client = self.clients.get_mut(client_id).unwrap();
        client.snake.push_front(new_head);
        client.snake.pop_back();

        MoveResult::Moved
    }

    pub fn remove_client(&mut self, client_id: &Uuid) {
        if let Some(client) = self.clients.remove(client_id) {
            for p in client.snake.iter() {
                self.occupied.set(idx(p), false);
            }
        }
    }
}
