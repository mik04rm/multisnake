use macroquad::prelude::*;
use multisnake_shared::{GRID_H, GRID_W, Pos};
use std::collections::VecDeque;

pub const CELL_SIZE: f32 = 20.0;
pub const WINDOW_W: f32 = GRID_W as f32 * CELL_SIZE;
pub const WINDOW_H: f32 = GRID_H as f32 * CELL_SIZE;

static DARKRED: Color = Color::new(0.5, 0.0, 0.0, 1.0);
static DARKYELLOW: Color = Color::new(0.5, 0.5, 0.0, 1.0);

pub fn draw_grid() {
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
}

pub fn draw_snake(snake: &VecDeque<Pos>, is_me: bool, is_ghost: bool) {
    for (i, p) in snake.iter().enumerate() {
        let x = p.x as f32 * CELL_SIZE;
        let y = p.y as f32 * CELL_SIZE;

        let color = if is_ghost {
            if i == 0 { YELLOW } else { DARKYELLOW }
        } else if is_me {
            if i == 0 { GREEN } else { DARKGREEN }
        } else {
            if i == 0 { RED } else { DARKRED }
        };

        draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, color);
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
