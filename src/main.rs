extern crate enigo;
use enigo::{Enigo, MouseControllable, MouseButton};
use std::thread;
use std::time::Duration;

fn main() {
    let wait_time = Duration::from_secs(2);
    let mut enigo = Enigo::new();
    enigo.mouse_move_to(500, 500);
    thread::sleep(wait_time);
}
