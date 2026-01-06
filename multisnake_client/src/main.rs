mod draw;
mod room_connection;
mod room_state;
mod tui;

use macroquad::prelude::*;
use multisnake_shared::SnakeMessage;
use room_state::RoomState;
use std::time::{Duration, Instant};

const BACK_TUI_DELAY_MS: u64 = 5000;

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        let maybe_selected_room = tokio_runtime.block_on(async { tui::run_room_selector().await });

        let selected_room = match maybe_selected_room {
            Ok(Some(room)) => room,
            Ok(None) => {
                println!("Exiting...");
                return;
            }
            Err(e) => {
                eprintln!("TUI error: {}", e);
                return;
            }
        };

        // Channels for communication between server and client
        let (from_client_tx, from_client_rx) =
            tokio::sync::mpsc::unbounded_channel::<SnakeMessage>();
        let (from_server_tx, from_server_rx) = std::sync::mpsc::channel();

        tokio_runtime.spawn(async move {
            room_connection::run(
                format!("ws://127.0.0.1:8080/room/{}", selected_room),
                from_client_rx,
                from_server_tx,
            )
            .await;
        });

        let mut room_state = RoomState::new();
        let mut death_time: Option<Instant> = None;

        loop {
            if room_state.alive {
                if let Some((dx, dy)) = room_state.handle_input() {
                    let _ = from_client_tx.send(SnakeMessage::MoveIntent { dx, dy });
                }
            }

            // Process incoming messages from server
            while let Ok(msg) = from_server_rx.try_recv() {
                if matches!(msg, SnakeMessage::TickUpdate { .. }) {
                    room_state.snapshot_state();
                }
                room_state.process_message(msg);
            }

            // Drawing
            if !room_state.alive && room_state.my_id.is_some() {
                draw::draw_game_finished();
                match death_time {
                    None => {
                        death_time = Some(Instant::now());
                    }
                    Some(time) => {
                        if time.elapsed() >= Duration::from_millis(BACK_TUI_DELAY_MS) {
                            // TODO: come back to tui in nice way
                            // (can just pump frames without closing macroquad window if it is hard)
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

            let mut t = 1.0; // TODO: xddd
            if let Some(tick_duration_ms) = room_state.tick_duration_ms {
                t = (elapsed as f32 / tick_duration_ms as f32).min(1.0);
            }

            if let Some(snake) = &room_state.my_snake {
                draw::draw_snake(
                    &snake.segments,
                    room_state.prev_my_snake.as_ref(),
                    t,
                    true,
                    room_state.ghosts.contains(&room_state.my_id.unwrap()),
                );
            }

            for (id, snake) in room_state.other_snakes.iter() {
                let prev_segments = room_state.prev_other_snakes.get(id);

                draw::draw_snake(
                    &snake.segments,
                    prev_segments,
                    t,
                    false,
                    room_state.ghosts.contains(id),
                );
            }
            if let Some(food_pos) = room_state.food {
                draw::draw_food(Some(food_pos));
            }

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
