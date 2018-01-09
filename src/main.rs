extern crate enigo;
use enigo::{Enigo, MouseControllable, MouseButton};
use std::thread;
use std::time::Duration;

fn main() {
    // Coordinates currently assume a resolution of 1920x1200. At some point,
    // we'll need to find a way to calculate the position of the easel relative
    // to the input resolution, but at the moment this works.
    
    // Portrait dimensions. Interestingly enough, it seems that the portrait is
    // slightly smaller than the landscape portrait.
    let lx = 431;
    let rx = 928;
    let uy = 110;
    let dy = 820;

    // Landscape dimensions.
    /*
    let lx = 318;
    let rx = 1028;
    let uy = 320;
    let dy = 820;
    */

    // Assume that the game runs at 60Hz, we should refresh every 16ms. Hence,
    // let's wait two frames during each operation.
    let wait_time = Duration::from_millis(32);
    let mut enigo = Enigo::new();

    // This should draw a black line across the left of the easel.
    enigo.mouse_move_to(lx, uy);
    thread::sleep(wait_time);
    enigo.mouse_down(MouseButton::Left);
    thread::sleep(wait_time);
    enigo.mouse_move_to(lx, dy);
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
    enigo.mouse_move_to(rx, uy);
    thread::sleep(wait_time);
    enigo.mouse_down(MouseButton::Left);
    thread::sleep(wait_time);
    enigo.mouse_move_to(rx, dy);
    thread::sleep(wait_time);
    enigo.mouse_up(MouseButton::Left);
    thread::sleep(wait_time);
}
