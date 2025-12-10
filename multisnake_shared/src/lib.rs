use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const GRID_W: i32 = 20;
pub const GRID_H: i32 = 20;
pub const MOVE_DELAY_SEC: f32 = 0.2;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum SnakeMessage {
    Init {
        client_id: Uuid,
    },
    Move {
        client_id: Uuid,
        dx: i32,
        dy: i32,
    },
    InitSnake {
        client_id: Uuid,
        segments: VecDeque<Pos>,
    },
    InitSnakeResponse {
        client_id: Uuid,
        segments: VecDeque<Pos>,
    },
    Dead {
        client_id: Uuid,
    },
    AteFood {
        client_id: Uuid,
    },
    FoodPos {
        pos: Pos,
    },
}
