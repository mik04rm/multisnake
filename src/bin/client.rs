use macroquad::prelude::*;
use serde::Serialize;
// use std::thread;
// use tokio::runtime::Runtime;
// use tokio::sync::mpsc;
// use tokio_tungstenite::{connect_async, tungstenite::Message};
// use futures_util::{SinkExt, StreamExt};

const GRID_W: i32 = 20;
const GRID_H: i32 = 20;
const CELL_SIZE: f32 = 28.0;
const WINDOW_W: f32 = GRID_W as f32 * CELL_SIZE;
const WINDOW_H: f32 = GRID_H as f32 * CELL_SIZE;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn to_screen(&self) -> Vec2 {
        vec2(self.x as f32 * CELL_SIZE, self.y as f32 * CELL_SIZE)
    }
}

#[macroquad::main("Snake + Tokio")]
async fn main() {
    // let (tx, mut rx) = mpsc::unbounded_channel::<Dir>();

    // thread::spawn(move || {
    //     let rt = Runtime::new().unwrap();

    //     rt.block_on(async move {

    //         let url = "ws://127.0.0.1:8080";
    //         info!("Connected to WebSocket server");
    //         let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    //         let (mut write, mut read) = ws_stream.split();

    //         tokio::spawn(async move {
    //             while let Some(direction) = rx.recv().await {
    //                 // Process the received direction
    //                 println!("Received direction from Macroquad: {:?}", direction);
    //                 let json = serde_json::to_string(&direction).unwrap();
    //                 write.send(Message::Text(json.into())).await.unwrap();
    //             }
    //         });

    //         loop {
    //             if let Some(msg) = read.next().await {
    //                 match msg {
    //                     Ok(Message::Text(txt)) => {
    //                         println!("Received from server: {}", txt);
    //                     }
    //                     Ok(_) => {}
    //                     Err(e) => {
    //                         eprintln!("WebSocket error: {}", e);
    //                         break;
    //                     }
    //                 }
    //             }
    //         }
    //     });
    // });

    request_new_screen_size(WINDOW_W, WINDOW_H + 40.0);
    next_frame().await;

    let midx = GRID_W / 2;
    let midy = GRID_H / 2;

    let mut snake = vec![
        Pos { x: midx, y: midy },
        Pos {
            x: midx - 1,
            y: midy,
        },
        Pos {
            x: midx - 2,
            y: midy,
        },
    ];

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


            let head = snake[0];
            let new_head = Pos {
                x: (head.x + dir.0 + GRID_W) % GRID_W,
                y: (head.y + dir.1 + GRID_H) % GRID_H,
            };

            snake.insert(0, new_head);
            snake.pop();

            // tx.send(dir).unwrap();
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

        for (i, s) in snake.iter().enumerate() {
            let pos = s.to_screen();
            draw_rectangle(
                pos.x,
                pos.y,
                CELL_SIZE,
                CELL_SIZE,
                if i == 0 { GREEN } else { DARKGREEN },
            );
        }

        next_frame().await;
    }
}
