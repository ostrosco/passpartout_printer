#[macro_use]
extern crate serde_derive;
extern crate enigo;
extern crate failure;

pub mod window;

use window::Window;
use enigo::{Enigo, MouseControllable, MouseButton};
use std::thread;
use std::time::Duration;


fn main() {
    // Coordinates currently assume a resolution of 1920x1200. At some point,
    // we'll need to find a way to calculate the position of the easel relative
    // to the input resolution, but at the moment this works.

    let win = match Window::new("coords.json".to_string()) {
        Ok(w) => w,
        Err(e) => panic!("Error loading config: {}", e.cause()),
    };
    
    // Assume that the game runs at 60Hz, we should refresh every 16ms. Hence,
    // let's wait two frames during each operation.
    let wait_time = Duration::from_millis(32);
    let mut enigo = Enigo::new();
    let (ul, lr) = win.portrait_bounds;

    // This should draw a black line across the left of the easel.
    enigo.mouse_move_to(ul.0, ul.1);
    thread::sleep(wait_time);
    enigo.mouse_down(MouseButton::Left);
    thread::sleep(wait_time);
    enigo.mouse_move_to(ul.0, lr.1);
    thread::sleep(wait_time);
    enigo.mouse_up(MouseButton::Left);
    thread::sleep(wait_time);

    // Let's change the color to everglade.
    enigo.mouse_move_to(735, 930);
    thread::sleep(wait_time);
    enigo.mouse_down(MouseButton::Left);
    thread::sleep(wait_time);
    enigo.mouse_up(MouseButton::Left);
    thread::sleep(wait_time);

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
