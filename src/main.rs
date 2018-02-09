extern crate enigo;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate glutin;
extern crate image;
#[macro_use]
extern crate serde_derive;

pub mod easel;
pub mod image_trans;

use enigo::Enigo;
use image_trans::size_to_easel;
use failure::Error;
use std::env;
use std::time::Duration;
use std::thread;
use std::u64;
use std::sync::mpsc;

use easel::{Color, Easel, Orientation};
use image::Pixel;

fn app() -> Result<(), Error> {
    let (tx, rx) = mpsc::channel();

    // A simple event loop to search for the escape key to pause drawing.
    thread::spawn(move || {
        use glutin::{DeviceEvent, ElementState, Event, VirtualKeyCode};
        let mut events_loop = glutin::EventsLoop::new();
        events_loop.run_forever(|event| {
            if let Event::DeviceEvent { event, .. } = event {
                if let DeviceEvent::Key(key) = event {
                    if let (
                        Some(VirtualKeyCode::Escape),
                        ElementState::Released,
                    ) = (key.virtual_keycode, key.state)
                    {
                        tx.send(()).unwrap();
                    }
                }
            }
            glutin::ControlFlow::Continue
        });
    });

    let easel_config = String::from("coords.json");
    let picture: String = env::args().nth(1).unwrap();
    let duration: u64 = match env::args().nth(2) {
        Some(v) => v.parse().unwrap(),
        None => 10_u64,
    };
    let wait_time = Duration::from_millis(duration);
    let mut enigo = Enigo::new();
    let mut easel = Easel::new(easel_config)?;
    let mut image = size_to_easel(&image::open(picture)?, &easel).to_rgb();
    let brush_wait = Duration::from_millis(32);
    easel.change_brush_size(0, &mut enigo, &brush_wait);

    let (size_x, size_y) = image.dimensions();
    if (size_x > size_y && easel.orientation == Orientation::Portrait)
        || (size_y > size_x && easel.orientation == Orientation::Landscape)
    {
        easel.change_orientation(&mut enigo, &wait_time);
    }

    let mut current_color = easel.current_color;
    let mut start_x = 0;
    let mut start_y = 0;
    let mut paused = false;
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        // Loop to handle pausing drawing so the user can actually get control
        // of their machine again.
        loop {
            if let Ok(()) = rx.try_recv() {
                paused = !paused;
                if paused {
                    println!("Pausing printing.");
                } else {
                    println!("Resuming printing.");
                }
            }
            if !paused {
                break;
            }
        }

        let rgb = pixel.to_rgb().data;
        let rgb = (rgb[0], rgb[1], rgb[2]);
        let closest_color = Color::find_closest_color(rgb);

        // If we've hit the end of a row, draw the rest of the row before
        // moving on to the next row.
        if y > start_y {
            easel.draw_line(
                (start_x as i32, start_y as i32),
                (size_x as i32, start_y as i32),
                &current_color,
                &mut enigo,
                &wait_time,
            )?;
            start_x = x;
            start_y = y;
            current_color = closest_color;
        }

        // If there's a color change, draw the line up to this pixel and stop.
        if closest_color != current_color {
            easel.draw_line(
                (start_x as i32, start_y as i32),
                (x as i32 - 1, y as i32),
                &current_color,
                &mut enigo,
                &wait_time,
            )?;
            start_x = x;
            start_y = y;
            current_color = closest_color;
        }
    }
    Ok(())
}

fn main() {
    app().unwrap();
}
