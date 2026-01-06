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
    pub my_id: Option<Uuid>,
    pub my_snake: Option<Snake>,
    pub other_snakes: HashMap<Uuid, Snake>,
    pub alive: bool,
    pub food: Option<Pos>,
    pub ghosts: Vec<Uuid>,

    pub prev_my_snake: Option<VecDeque<Pos>>,
    pub prev_other_snakes: HashMap<uuid::Uuid, VecDeque<Pos>>,
    pub last_update_time: Instant,
    pub tick_duration_ms: Option<u32>,
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            my_id: None,
            my_snake: None,
            other_snakes: HashMap::new(),
            alive: true,
            food: None,
            ghosts: Vec::new(),

            prev_my_snake: None,
            prev_other_snakes: HashMap::new(),
            last_update_time: Instant::now(),

            tick_duration_ms: None,
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
            SnakeMessage::OnJoin {
                my_id,
                snakes,
                tick_duration_ms,
            } => {
                self.my_id = Some(my_id);
                self.alive = true;
                self.tick_duration_ms = Some(tick_duration_ms);

                assert!(
                    self.my_snake.is_none(),
                    "Received OnJoin but my_snake is already set!"
                );
                assert!(
                    self.other_snakes.is_empty(),
                    "Received OnJoin but other_snakes is not empty!"
                );

                for (id, segments) in snakes {
                    if id == my_id {
                        self.my_snake = Some(Snake::new(segments));
                    } else {
                        self.other_snakes.insert(id, Snake::new(segments));
                    }
                }
            }
            SnakeMessage::TickUpdate {
                moves,
                food,
                deaths,
                eaters,
                new_snakes,
                ghosts,
            } => {
                self.food = Some(food);

                // Add new clients snakes
                for (id, segments) in new_snakes {
                    if Some(id) != self.my_id {
                        self.other_snakes.insert(id, Snake::new(segments));
                    }
                }

                // Process deaths
                for id in deaths {
                    if Some(id) == self.my_id {
                        println!("You died!");
                        self.alive = false;
                    }
                    self.other_snakes.remove(&id);
                }

                // Process moves
                for (id, (dx, dy)) in moves {
                    let growing = eaters.contains(&id); // TODO opt, can change message structure

                    if Some(id) == self.my_id {
                        if let Some(snake) = &mut self.my_snake {
                            snake.apply_move(dx, dy, growing);
                        }
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
        if let Some(snake) = &self.my_snake {
            self.prev_my_snake = Some(snake.segments.clone());
        }

        self.prev_other_snakes.clear();
        for (id, snake) in &self.other_snakes {
            self.prev_other_snakes.insert(*id, snake.segments.clone());
        }

        self.last_update_time = std::time::Instant::now();
    }
}
