extern crate enigo;
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure_derive;

pub mod easel;

use easel::{Color, Easel};
use enigo::{Enigo, MouseButton, MouseControllable};
use std::thread;
use std::time::Duration;

fn main() {
    // Coordinates currently assume a resolution of 1920x1200. At some point,
    // we'll need to find a way to calculate the position of the easel relative
    // to the input resolution, but at the moment this works.

    let mut easel = match Easel::new("coords.json".to_string()) {
        Ok(e) => e,
        Err(e) => panic!("Error loading config: {}", e.cause()),
    };

    // Assume that the game runs at 60Hz, we should refresh every 16ms. Hence,
    // let's wait two frames during each operation.
    let wait_time = Duration::from_millis(32);
    let mut enigo = Enigo::new();

    easel.change_orientation(&mut enigo, &wait_time);
    let (ul, lr) = easel.get_bounds();

    // Next, shrink the brush size to 3.
    easel.change_brush_size(3, &mut enigo, &wait_time);

    // This is an estimate of the brush size in pixels.
    // TODO: I need to go through and find a sane way of estimating this.
    let brush_pixel = 38;

    let col_step = (lr.0 - ul.0) / 24;
    let colors = vec![
        Color::Black,
        Color::Grey,
        Color::White,
        Color::DarkBrown,
        Color::Brown,
        Color::LightBrown,
        Color::DarkRed,
        Color::Red,
        Color::Pink,
        Color::Orange,
        Color::DarkYellow,
        Color::Yellow,
        Color::DarkGreen,
        Color::Green,
        Color::LightGreen,
        Color::DarkBlue,
        Color::Blue,
        Color::LightBlue,
        Color::DarkIndigo,
        Color::Indigo,
        Color::LightIndigo,
        Color::DarkViolet,
        Color::Violet,
        Color::LightViolet,
    ];
    for (col, color) in colors.iter().enumerate() {
        easel.change_color(color, &mut enigo, &wait_time);
        let col_coords = ul.0 + col_step * (col as i32 + 1) + brush_pixel;
        enigo.mouse_move_to(col_coords, ul.1);
        thread::sleep(wait_time);
        enigo.mouse_down(MouseButton::Left);
        thread::sleep(wait_time);
        enigo.mouse_move_to(col_coords, lr.1);
        thread::sleep(wait_time);
        enigo.mouse_up(MouseButton::Left);
        thread::sleep(wait_time);
    }

    let pixel_colors = vec![
        0xff_ff_ff, 0xc0_c0_c0, 0x80_80_80, 0x00_00_00, 0xff_00_00, 0x80_00_00, 0xff_ff_00,
        0x80_80_00, 0x00_ff_00, 0x00_80_00, 0x00_ff_ff, 0x00_80_80, 0x00_00_ff, 0x00_00_80,
        0xff_00_ff, 0x80_00_80,
    ];
    for (col, color) in pixel_colors.iter().enumerate() {
        let coords = (((col as i32 + 1) * 30), 300);
        easel.draw_pixel(coords, *color, &mut enigo, &wait_time).unwrap();
    }
}