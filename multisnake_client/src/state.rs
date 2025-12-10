use macroquad::prelude::{KeyCode, is_key_pressed};
use multisnake_shared::{GRID_H, GRID_W, Pos, SnakeMessage};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

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
    pub fn new_empty() -> Self {
        Self {
            segments: VecDeque::new(),
            growing: false,
        }
    }
}

pub struct GameState {
    pub my_id: Option<Uuid>,
    pub my_snake: Snake,
    pub other_snakes: HashMap<Uuid, Snake>,
    pub alive: bool,
    pub next_move_dir: (i32, i32),
    pub food: Option<Pos>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            my_id: None,
            my_snake: Snake::new_empty(),
            other_snakes: HashMap::new(),
            alive: true,
            next_move_dir: (1, 0),
            food: None,
        }
    }

    pub fn handle_input(&mut self) {
        self.next_move_dir = if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            (0, -1)
        } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            (0, 1)
        } else if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            (-1, 0)
        } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            (1, 0)
        } else {
            self.next_move_dir
        };
    }

    pub fn move_snake(&mut self) {
        let head = self.my_snake.segments[0];
        let (dx, dy) = self.next_move_dir;

        let new_head = Pos {
            x: (head.x + dx + GRID_W) % GRID_W,
            y: (head.y + dy + GRID_H) % GRID_H,
        };
        self.my_snake.segments.push_front(new_head);
        if self.my_snake.growing {
            self.my_snake.growing = false;
        } else {
            self.my_snake.segments.pop_back();
        }
    }

    // Returns a message to send to server, if any
    pub fn process_message(&mut self, msg: SnakeMessage) -> Option<SnakeMessage> {
        match msg {
            SnakeMessage::InitSnake {
                client_id,
                segments,
            } => {
                // The first InitSnake we receive is for this client
                if self.my_id.is_none() {
                    self.my_id = Some(client_id);
                    self.my_snake = Snake {
                        segments,
                        growing: false,
                    };
                } else {
                    self.other_snakes.insert(client_id, Snake::new(segments));
                }
                Some(SnakeMessage::InitSnakeResponse {
                    client_id: self.my_id.unwrap(),
                    segments: self.my_snake.segments.clone(),
                })
            }
            SnakeMessage::InitSnakeResponse {
                client_id,
                segments,
            } => {
                self.other_snakes.insert(client_id, Snake::new(segments));
                None
            }
            SnakeMessage::Move { client_id, dx, dy } => {
                if Some(client_id) != self.my_id {
                    if let Some(snake) = self.other_snakes.get_mut(&client_id) {
                        let head = snake.segments[0];
                        let new_head = Pos {
                            x: (head.x + dx + GRID_W) % GRID_W,
                            y: (head.y + dy + GRID_H) % GRID_H,
                        };
                        snake.segments.push_front(new_head);
                        if snake.growing {
                            snake.growing = false;
                        } else {
                            snake.segments.pop_back();
                        }
                    }
                }
                None
            }
            SnakeMessage::Dead { client_id } => {
                if Some(client_id) == self.my_id {
                    self.alive = false;
                    println!("I died.");
                } else {
                    self.other_snakes.remove(&client_id);
                }
                None
            }
            SnakeMessage::AteFood { client_id } => {
                if client_id == self.my_id.unwrap() {
                    println!("I ate food!");
                    self.my_snake.growing = true;
                } else {
                    println!("Client {:?} ate food!", client_id);
                    let snake = self.other_snakes.get_mut(&client_id).unwrap();
                    snake.growing = true;
                }
                None
            }
            SnakeMessage::FoodPos { pos } => {
                println!("New food position: {:?}", pos);
                self.food = Some(pos);
                None
            }
            _ => None, // TODO
        }
    }
}
