mod draw;
mod network;
mod state;
mod tui;

use macroquad::prelude::*;
use multisnake_shared::{self, SnakeMessage};
use state::GameState;

#[macroquad::main(window_conf)]
async fn main() {
    // Channels (queues) for communication between server and client
    let (from_client_tx, from_client_rx) = tokio::sync::mpsc::unbounded_channel::<SnakeMessage>();
    let (from_server_tx, from_server_rx) = std::sync::mpsc::channel();

    network::spawn_network_thread(
        "ws://127.0.0.1:8080".to_string(),
        from_client_rx,
        from_server_tx,
    );

    // TODO: tui
    // tui::spawn_tui();

    let mut game_state = GameState::new();
    let mut move_timer = 0.0;

    loop {
        let dt = get_frame_time();
        move_timer += dt;

        game_state.handle_input();

        // Process incoming messages from server
        while let Ok(msg) = from_server_rx.try_recv() {
            if let Some(my_response) = game_state.process_message(msg) {
                let _ = from_client_tx.send(my_response);
            }
        }

        // Movement tick
        if game_state.alive && move_timer >= multisnake_shared::MOVE_DELAY_SEC {
            move_timer = 0.0;
            game_state.move_snake();
            let _ = from_client_tx.send(SnakeMessage::Move {
                client_id: game_state.my_id.unwrap(),
                dx: game_state.next_move_dir.0,
                dy: game_state.next_move_dir.1,
            });
        }

        if !game_state.alive {
            draw::draw_game_over();
            next_frame().await;
            continue;
        }

        // Drawing
        clear_background(BLACK);
        draw::draw_grid();
        draw::draw_snake(&game_state.my_snake.segments, true);

        for snake in game_state.other_snakes.values() {
            draw::draw_snake(&snake.segments, false);
        }

        draw::draw_food(game_state.food);

        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Snake Client".to_string(),
        window_width: draw::WINDOW_W as i32,
        window_height: draw::WINDOW_H as i32,
        high_dpi: true,
        ..Default::default()
    }
}
