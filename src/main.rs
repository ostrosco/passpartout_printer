#[macro_use]
extern crate serde_derive;
extern crate enigo;
extern crate failure;

pub mod easel;

use easel::Easel;
use enigo::{Enigo, MouseControllable, MouseButton};
use std::thread;
use std::time::Duration;

fn change_color(enigo: &mut Enigo, wait_time: &Duration, x: i32, y: i32) -> () {
    // Let's change the color to everglade.
    enigo.mouse_move_to(x, y);
    thread::sleep(*wait_time);
    enigo.mouse_down(MouseButton::Left);
    thread::sleep(*wait_time);
    enigo.mouse_up(MouseButton::Left);
    thread::sleep(*wait_time);
}

fn main() {
    // Coordinates currently assume a resolution of 1920x1200. At some point,
    // we'll need to find a way to calculate the position of the easel relative
    // to the input resolution, but at the moment this works.

    let easel = match Easel::new("coords.json".to_string()) {
        Ok(e) => e,
        Err(e) => panic!("Error loading config: {}", e.cause()),
    };
    
    // Assume that the game runs at 60Hz, we should refresh every 16ms. Hence,
    // let's wait two frames during each operation.
    let wait_time = Duration::from_millis(32);
    let mut enigo = Enigo::new();
    let (ul, lr) = easel.portrait_bounds;
    let (everglade_x, everglade_y) = (easel.color_start.0 + easel.color_col_step * 4,
                                      easel.color_start.1);
    let (pink_x, pink_y) = (easel.color_start.0 + easel.color_col_step * 2,
                            easel.color_start.1 + easel.color_row_step * 2);
    
    // Let's first change the color to pink.
    change_color(&mut enigo, &wait_time, pink_x, pink_y);

    // This should draw a black line across the left of the easel.
    enigo.mouse_move_to(ul.0, ul.1);
    thread::sleep(wait_time);
    enigo.mouse_down(MouseButton::Left);
    thread::sleep(wait_time);
    enigo.mouse_move_to(ul.0, lr.1);
    thread::sleep(wait_time);
    enigo.mouse_up(MouseButton::Left);
    thread::sleep(wait_time);

    // Now, change the color to everglade.
    change_color(&mut enigo, &wait_time, everglade_x, everglade_y);

    // Now, draw an everglade line across the right of an easel.
    enigo.mouse_move_to(lr.0, ul.1);
    thread::sleep(wait_time);
    enigo.mouse_down(MouseButton::Left);
    thread::sleep(wait_time);
    enigo.mouse_move_to(lr.0, lr.1);
    thread::sleep(wait_time);
    enigo.mouse_up(MouseButton::Left);
    thread::sleep(wait_time);
}
