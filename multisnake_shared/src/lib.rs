use std::collections::{HashMap, VecDeque};

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
    /// Sent to a client immediately upon connection
    InitGame {
        my_id: Uuid,
        // Snapshot of all existing snakes
        snakes: HashMap<Uuid, VecDeque<Pos>>,
        food: Pos,
    },
    /// The heartbeat of the game (sent every ~150ms)
    TickUpdate {
        // Only sends changes in position (dx, dy) for living snakes
        moves: HashMap<Uuid, (i32, i32)>,
        food: Pos,
        // List of IDs that died this tick
        deaths: Vec<Uuid>,
        // List of IDs that grew this tick
        eaters: Vec<Uuid>,
        // Full body segments of players who joined this tick
        new_snakes: HashMap<Uuid, VecDeque<Pos>>,

        ghosts: Vec<Uuid>,
    },
    /// Client -> Server: "I want to go this way"
    MoveIntent { dx: i32, dy: i32 },
}
