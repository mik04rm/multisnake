use macroquad::prelude::*;
use multisnake_shared::{GRID_H, GRID_W, Pos};
use std::collections::VecDeque;

pub const CELL_SIZE: f32 = 15.0;
pub const WINDOW_W: f32 = GRID_W as f32 * CELL_SIZE;
pub const WINDOW_H: f32 = GRID_H as f32 * CELL_SIZE;

const DARKRED: Color = Color::new(0.5, 0.0, 0.0, 1.0);
const DARKYELLOW: Color = Color::new(0.5, 0.5, 0.0, 1.0);

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn draw_grid() {
    for x in 1..GRID_W {
        draw_line(
            x as f32 * CELL_SIZE,
            0.0,
            x as f32 * CELL_SIZE,
            WINDOW_H,
            1.0,
            DARKGRAY,
        );
    }
    for y in 1..GRID_H {
        draw_line(
            0.0,
            y as f32 * CELL_SIZE,
            WINDOW_W,
            y as f32 * CELL_SIZE,
            1.0,
            DARKGRAY,
        );
    }
}

pub fn draw_snake(
    snake: &VecDeque<Pos>,
    prev_snake: Option<&VecDeque<Pos>>,
    t: f32,
    is_me: bool,
    is_ghost: bool,
) {
    for (i, current_pos) in snake.iter().enumerate().rev() {
        let is_head = i == 0;
        let is_tail = i == snake.len() - 1;

        let color = match (is_ghost, is_me, is_head) {
            (true, _, true) => YELLOW,
            (true, _, false) => DARKYELLOW,
            (_, true, true) => GREEN,
            (_, true, false) => DARKGREEN,
            (_, _, true) => RED,
            (_, _, false) => DARKRED,
        };

        // Visually fills the gap when the snake is changing direction.
        if is_tail {
            draw_rectangle(
                current_pos.x as f32 * CELL_SIZE,
                current_pos.y as f32 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
                color,
            );
        }

        let mut x = current_pos.x as f32;
        let mut y = current_pos.y as f32;

        if (is_head || is_tail)
            && let Some(prev) = prev_snake
        {
            let prev_pos = prev.get(i).unwrap_or(current_pos);
            x = lerp(prev_pos.x as f32, x, t);
            y = lerp(prev_pos.y as f32, y, t);
        }

        draw_rectangle(x * CELL_SIZE, y * CELL_SIZE, CELL_SIZE, CELL_SIZE, color);
    }
}

pub fn draw_food(food: Option<Pos>) {
    if let Some(food_pos) = food {
        let x = food_pos.x as f32 * CELL_SIZE;
        let y = food_pos.y as f32 * CELL_SIZE;
        draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, DARKYELLOW);
    }
}
pub fn draw_game_over() {
    clear_background(BLACK);
    let text = "Game finished!";
    let font_size = 30.0;
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(
        text,
        (WINDOW_W - dims.width) / 2.0,
        (WINDOW_H - dims.height) / 2.0,
        font_size,
        WHITE,
    );
}
