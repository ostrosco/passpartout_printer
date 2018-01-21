extern crate enigo;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;

pub mod easel;
pub mod image_trans;

use enigo::Enigo;
use std::env;
use std::time::Duration;

fn main() {
    let picture: String = env::args().nth(1).unwrap();

    // Assume that the game runs at 60Hz, we should refresh every 16ms. Hence,
    // let's wait two frames during each operation.
    let wait_time = Duration::from_millis(32);
    let mut enigo = Enigo::new();
    image_trans::draw_image(
        &picture,
        String::from("coords.json"),
        &mut enigo,
        &wait_time,
    ).unwrap();
}
