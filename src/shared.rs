use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const GRID_W: i32 = 20;
pub const GRID_H: i32 = 20;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

// ===== Client → Server =====
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum SnakeMessage {
    CreateRoom { room_name: String },
    JoinRoom { room_name: String },
    LeaveRoom { room_name: String },
    // Move { dx: i32, dy: i32 },
    // Chat { text: String },
    Init { client_id: Uuid },
    RoomsList { rooms: Vec<RoomInfo> },
    PlayerJoined { client_id: Uuid, room: String },
    PlayerLeft { client_id: Uuid, room: String },
    Move { client_id: Uuid, dx: i32, dy: i32 },
    Chat { client_id: Uuid, text: String },
    InitSnake { client_id: Uuid, segments: VecDeque<Pos> },
    InitSnakeResponse { client_id: Uuid, segments: VecDeque<Pos> },
    Dead { client_id: Uuid },
}

// // ===== Server → Client =====
// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(tag = "type", content = "data")]
// pub enum Message {
    
// }

// Room info for lobby display
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomInfo {
    pub name: String,
    pub player_count: usize,
}
