use macroquad::prelude::{KeyCode, is_key_pressed};
use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};
use uuid::Uuid;

use multisnake_shared::{Pos, SnakeMessage};

pub struct Snake {
    pub segments: VecDeque<Pos>,
    pub growing: bool,
}

impl Snake {
    pub fn new(segments: VecDeque<Pos>) -> Self {
        Self {
            segments,
            growing: false,
        }
    }

    pub fn apply_move(&mut self, dx: i32, dy: i32, growing: bool) {
        if let Some(head) = self.segments.front() {
            let new_head = Pos {
                x: head.x + dx,
                y: head.y + dy,
            };

            self.segments.push_front(new_head);

            if growing {
                self.growing = false;
            } else {
                self.segments.pop_back();
            }
        }
    }
}

pub struct RoomState {
    pub my_id: Uuid,
    pub my_snake: Snake,
    pub other_snakes: HashMap<Uuid, Snake>,
    pub alive: bool,
    pub food: Pos,
    pub ghosts: Vec<Uuid>,

    pub prev_my_snake: Option<VecDeque<Pos>>,
    pub prev_other_snakes: HashMap<uuid::Uuid, VecDeque<Pos>>,
    pub last_update_time: Instant,
    pub tick_duration_ms: u32,
}

impl RoomState {
    pub fn new(my_id: Uuid, snakes: HashMap<Uuid, VecDeque<Pos>>, tick_duration_ms: u32, food: Pos) -> Self {
        let mut my_snake = Snake::new(VecDeque::new());
        let mut other_snakes = HashMap::new();

        for (id, segments) in snakes {
            if id == my_id {
                my_snake = Snake::new(segments);
            } else {
                other_snakes.insert(id, Snake::new(segments));
            }
        }

        Self {
            my_id,
            my_snake,
            other_snakes,
            alive: true,
            food,
            ghosts: Vec::new(),

            prev_my_snake: None,
            prev_other_snakes: HashMap::new(),
            last_update_time: Instant::now(),

            tick_duration_ms,
        }
    }

    pub fn handle_input(&self) -> Option<(i32, i32)> {
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            Some((0, -1))
        } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            Some((0, 1))
        } else if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            Some((-1, 0))
        } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            Some((1, 0))
        } else {
            None
        }
    }

    pub fn process_message(&mut self, msg: SnakeMessage) {
        match msg {
            SnakeMessage::OnJoin { .. } => {}
            SnakeMessage::TickUpdate {
                moves,
                food,
                deaths,
                eaters,
                new_snakes,
                ghosts,
            } => {
                self.food = food;

                // Add new clients snakes
                for (id, segments) in new_snakes {
                    if id != self.my_id {
                        self.other_snakes.insert(id, Snake::new(segments));
                    }
                }

                // Process deaths
                for id in deaths {
                    if id == self.my_id {
                        println!("You died!");
                        self.alive = false;
                    }
                    self.other_snakes.remove(&id);
                }

                // Process moves
                for (id, (dx, dy)) in moves {
                    let growing = eaters.contains(&id); // TODO opt, can change message structure

                    if id == self.my_id {
                        self.my_snake.apply_move(dx, dy, growing);
                    } else if let Some(snake) = self.other_snakes.get_mut(&id) {
                        snake.apply_move(dx, dy, growing);
                    }
                }

                self.ghosts = ghosts;
            }
            _ => {}
        }
    }

    pub fn snapshot_state(&mut self) {
        self.prev_my_snake = Some(self.my_snake.segments.clone());
        self.prev_other_snakes.clear();
        for (id, snake) in &self.other_snakes {
            self.prev_other_snakes.insert(*id, snake.segments.clone());
        }

        self.last_update_time = std::time::Instant::now();
    }
}
