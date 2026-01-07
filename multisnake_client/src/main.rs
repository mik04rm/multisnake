mod draw;
mod room_connection;
mod room_state;
mod tui;

use clap::Parser;
use macroquad::prelude::*;
use multisnake_shared::SnakeMessage;
use room_state::RoomState;
use tokio::sync::mpsc;
use std::time::{Duration, Instant};

const BACK_TUI_DELAY_MS: u64 = 3000;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "127.0.0.1:4040")]
    server_addr: String,
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Args::parse();

    loop {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        let (tui_tx, mut tui_rx) = mpsc::unbounded_channel();
        let server_addr_clone = args.server_addr.clone();

        // Spawn TUI in a separate task
        tokio_runtime.spawn(async move {
            let result = tui::run_room_selector(&server_addr_clone).await.unwrap();
            let _ = tui_tx.send(result);
        });

        let selected_room;

        // Wait for room selection
        loop {
            if let Ok(maybe_room) = tui_rx.try_recv() {
                match maybe_room {
                    Some(room) => {
                        selected_room = room;
                        break;
                    }
                    None => {
                        println!("Exiting...");
                        return;
                    }
                }
            }
            clear_background(BLACK);
            draw_text("Please select a room in the terminal...", 20.0, 30.0, 30.0, WHITE);
            next_frame().await;
        }

        // Channels for communication between server and client
        let (from_client_tx, from_client_rx) =
            mpsc::unbounded_channel::<SnakeMessage>();
        let (from_server_tx, from_server_rx) = std::sync::mpsc::channel();

        let server_addr = args.server_addr.clone();

        tokio_runtime.spawn(async move {
            room_connection::run(
                format!("ws://{}/room/{}", server_addr, selected_room),
                from_client_rx,
                from_server_tx,
            )
            .await;
        });

        let Ok(SnakeMessage::OnJoin {
            my_id,
            snakes,
            tick_duration_ms,
            food,
        }) = from_server_rx.recv()
        else {
            eprintln!("Failed to receive OnJoin message from server");
            continue;
        };

        let mut room_state = RoomState::new(my_id, snakes, tick_duration_ms, food);
        let mut death_time: Option<Instant> = None;

        loop {
            if room_state.alive {
                if let Some((dx, dy)) = room_state.handle_input() {
                    let _ = from_client_tx.send(SnakeMessage::MoveIntent { dx, dy });
                }
            }

            // Process incoming messages from server
            while let Ok(msg) = from_server_rx.try_recv() {
                if let SnakeMessage::TickUpdate { .. } = msg {
                    room_state.snapshot_state();
                }
                room_state.process_message(msg);
            }

            // Drawing
            if !room_state.alive {
                draw::draw_game_finished();
                match death_time {
                    None => {
                        death_time = Some(Instant::now());
                    }
                    Some(time) => {
                        if time.elapsed() >= Duration::from_millis(BACK_TUI_DELAY_MS) {
                            break;
                        }
                    }
                }
                next_frame().await;
                continue;
            }

            clear_background(BLACK);
            draw::draw_grid();

            let elapsed = room_state.last_update_time.elapsed().as_millis();

            let interpol_t = (elapsed as f32 / room_state.tick_duration_ms as f32).min(1.0);

            draw::draw_snake(
                &room_state.my_snake.segments,
                room_state.prev_my_snake.as_ref(),
                interpol_t,
                true,
                room_state.ghosts.contains(&room_state.my_id),
            );

            for (id, snake) in room_state.other_snakes.iter() {
                let prev_segments = room_state.prev_other_snakes.get(id);

                draw::draw_snake(
                    &snake.segments,
                    prev_segments,
                    interpol_t,
                    false,
                    room_state.ghosts.contains(id),
                );
            }
            draw::draw_food(room_state.food);

            next_frame().await;
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "multisnake".to_string(),
        window_width: draw::WINDOW_W as i32,
        window_height: draw::WINDOW_H as i32,
        high_dpi: true,
        ..Default::default()
    }
}
