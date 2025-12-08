mod shared;

use futures_util::{SinkExt, StreamExt};
use macroquad::prelude::*;
use std::{collections::{HashMap, VecDeque}, hash::Hash, thread};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::stdout;

use crate::shared::{GRID_H, GRID_W, Pos, SnakeMessage};

pub fn spawn_tui() -> thread::JoinHandle<()> {
    thread::spawn(|| {
        // --- terminal init ---
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        // --- main loop ---
        loop {
            terminal
                .draw(|frame| {
                    let size = frame.size();
                    // draw something
                    // frame.render_widget(your_widget, size);
                })
                .unwrap();

            if event::poll(std::time::Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if key.code == crossterm::event::KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }

        // --- cleanup ---
        disable_raw_mode().unwrap();
        execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    })
}


const CELL_SIZE: f32 = 28.0;
const WINDOW_W: f32 = GRID_W as f32 * CELL_SIZE;
const WINDOW_H: f32 = GRID_H as f32 * CELL_SIZE;

impl Pos {
    fn to_screen(&self) -> Vec2 {
        vec2(self.x as f32 * CELL_SIZE, self.y as f32 * CELL_SIZE)
    }
}

#[macroquad::main("Snake + Tokio")]
async fn main() {
    let (from_client_tx, mut from_client_rx) = mpsc::unbounded_channel::<SnakeMessage>();
    let (from_server_tx, from_server_rx) = std::sync::mpsc::channel::<SnakeMessage>();

    thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async move {
            let url = "ws://127.0.0.1:8080";
            // info!("Connected to WebSocket server");
            let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
            println!("Connected to WebSocket server at {}", url);
            let (mut ws_tx, mut ws_rx) = ws_stream.split();

            tokio::spawn(async move {
                while let Some(message) = from_client_rx.recv().await {
                    // Process the received direction
                    println!("Received direction from Macroquad: {:?}", message);
                    let json = serde_json::to_string(&message).unwrap();
                    ws_tx.send(Message::Text(json.into())).await.unwrap();
                }
            });

            loop {
                if let Some(msg) = ws_rx.next().await {
                    match msg {
                        Ok(Message::Text(txt)) => {
                            println!("Received from server: {}", txt);
                            if let Ok(server_msg) = serde_json::from_str::<SnakeMessage>(&txt) {
                                println!("Parsed server message: {:?}", server_msg);
                                from_server_tx.send(server_msg).unwrap();
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            println!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    });

    // TODO better error handling

    let mut my_snake: VecDeque<Pos>;
    let mut other_snakes: HashMap<uuid::Uuid, VecDeque<Pos>> = HashMap::new();
    let my_id: uuid::Uuid;


    println!("Waiting for initial message from server...");
    let msg = from_server_rx.recv().unwrap();
    println!("Received initial message from_server_rx: {:?}", msg);

    match msg {
        SnakeMessage::InitSnake { segments, client_id } => {
            my_id = client_id;
            println!("Init snake from server: {:?}", segments);
            my_snake = segments;
        }
        _ => {
            panic!("Expected InitSnake message from server");
        }
    }

    println!("Initial snake: {:?}", my_snake);

    // let _ = spawn_tui();

    request_new_screen_size(WINDOW_W, WINDOW_H + 40.0);
    next_frame().await;

    let mut dir = (1, 0);
    let mut move_timer = 0.0;
    let move_delay = 0.20;

    loop {
        let dt = get_frame_time();

        let input_dir = if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            Some((0, -1))
        } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            Some((0, 1))
        } else if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            Some((-1, 0))
        } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            Some((1, 0))
        } else {
            None
        };

        if let Some(input_dir) = input_dir {
            dir = input_dir;
        }

        move_timer += dt;
        if move_timer >= move_delay {
            move_timer = 0.0;

            let head = my_snake[0];
            let new_head = Pos {
                x: (head.x + dir.0 + GRID_W) % GRID_W,
                y: (head.y + dir.1 + GRID_H) % GRID_H,
            };

            my_snake.push_front(new_head);
            my_snake.pop_back();

            let res = from_client_tx.send(SnakeMessage::Move {
                client_id: my_id,
                dx: dir.0,
                dy: dir.1,
            });
            println!("Sent move to server: {:?}", res);
        }

        let mut alive = true;

        while let Ok(msg) = from_server_rx.try_recv() {
            match msg {
                SnakeMessage::InitSnake {
                    client_id,
                    segments,
                } => {
                    let _ = from_client_tx.send(SnakeMessage::InitSnakeResponse {
                        client_id: my_id,
                        segments: my_snake.clone(),
                    });
                    other_snakes.insert(client_id, segments);
                }

                SnakeMessage::Move { client_id, dx, dy } => {
                    if let Some(snake) = other_snakes.get_mut(&client_id) {
                        let head = snake[0];
                        let new_head = Pos {
                            x: (head.x + dx + GRID_W) % GRID_W,
                            y: (head.y + dy + GRID_H) % GRID_H,
                        };
                        snake.push_front(new_head);
                        snake.pop_back();
                    }
                }

                SnakeMessage::InitSnakeResponse { client_id, segments } => {
                    other_snakes.insert(client_id, segments);
                }

                SnakeMessage::Dead { client_id } => {
                    if client_id == my_id {
                        println!("You are dead!");
                        alive = false;
                        // Handle player death (e.g., reset game or show game over screen)
                    } else {
                        other_snakes.remove(&client_id);
                    }
                }

                _ => {}
            }
        }

        if !alive {
            break;
        }

        clear_background(BLACK);

        for x in 0..=GRID_W {
            draw_line(
                x as f32 * CELL_SIZE,
                0.0,
                x as f32 * CELL_SIZE,
                WINDOW_H,
                1.0,
                GRAY,
            );
        }
        for y in 0..=GRID_H {
            draw_line(
                0.0,
                y as f32 * CELL_SIZE,
                WINDOW_W,
                y as f32 * CELL_SIZE,
                1.0,
                GRAY,
            );
        }

        for (i, s) in my_snake.iter().enumerate() {
            let pos = s.to_screen();
            draw_rectangle(
                pos.x,
                pos.y,
                CELL_SIZE,
                CELL_SIZE,
                if i == 0 { GREEN } else { DARKGREEN },
            );
        }

        for (_, segments) in other_snakes.iter() {
            println!("Drawing other snake: {:?}", segments);
            for (_, seg) in segments.iter().enumerate() {
                let pos = seg.to_screen();
                let color = RED;
                draw_rectangle(pos.x, pos.y, CELL_SIZE, CELL_SIZE, color);
            }
        }

        next_frame().await;
    }
}
