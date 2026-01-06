use macroquad::prelude::*;
use std::collections::VecDeque;

use multisnake_shared::{GRID_H, GRID_W, Pos};

pub const CELL_SIZE: f32 = 15.0;
pub const WINDOW_W: f32 = GRID_W as f32 * CELL_SIZE;
pub const WINDOW_H: f32 = GRID_H as f32 * CELL_SIZE;

const GRID_COLOR: Color = Color::from_rgba(40, 40, 40, 255);
const GHOST_HEAD_COLOR: Color = Color::from_rgba(217, 207, 39, 255);
const GHOST_BODY_COLOR: Color = Color::from_rgba(171, 163, 32, 255);
const ME_HEAD_COLOR: Color = Color::from_rgba(47, 189, 184, 255);
const ME_BODY_COLOR: Color = Color::from_rgba(37, 143, 139, 255);
const OTHER_HEAD_COLOR: Color = Color::from_rgba(219, 37, 55, 255);
const OTHER_BODY_COLOR: Color = Color::from_rgba(173, 28, 42, 255);
const FOOD_COLOR: Color = Color::from_rgba(104, 207, 91, 255);

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
            GRID_COLOR,
        );
    }
    for y in 1..GRID_H {
        draw_line(
            0.0,
            y as f32 * CELL_SIZE,
            WINDOW_W,
            y as f32 * CELL_SIZE,
            1.0,
            GRID_COLOR,
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
            (true, _, true) => GHOST_HEAD_COLOR,
            (true, _, false) => GHOST_BODY_COLOR,
            (_, true, true) => ME_HEAD_COLOR,
            (_, true, false) => ME_BODY_COLOR,
            (_, _, true) => OTHER_HEAD_COLOR,
            (_, _, false) => OTHER_BODY_COLOR,
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
        draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, FOOD_COLOR);
    }
}
pub fn draw_game_finished() {
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
