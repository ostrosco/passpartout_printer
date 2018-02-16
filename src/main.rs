#[macro_use]
extern crate clap;
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
pub mod colors;

use enigo::Enigo;
use image_trans::size_to_easel;
use failure::Error;
use std::time::Duration;
use std::thread;
use std::u64;
use std::sync::mpsc;
use image::imageops::ColorMap;
use image::imageops::dither;
use clap::App;

use easel::{Easel, Orientation};
use colors::{Palette, PaletteColor};
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

    let matches = App::new("Passpartout Printer")
        .version("0.1.0")
        .args_from_usage(
            "-w, --mouse-wait=[WAIT] 'Specify the time to wait between mouse actions'
            --enable-dither 'Enables dithering to reduce color banding but increase draw time'
            <IMAGE> 'Input image to use'")
        .get_matches();

    let easel_config = String::from("coords.json");
    let image_path: String = matches.value_of("IMAGE").unwrap().to_string();
    let mouse_wait: u64 = value_t!(matches, "mouse-wait", u64).unwrap_or(7);
    let enable_dither: bool = match matches.occurrences_of("enable-dither") {
        0 => false,
        _ => true,
    };
    println!("Printing to Passpartout with the following settings:");
    println!("-- image: {}", image_path);
    println!("-- mouse wait: {}", mouse_wait);
    println!("-- dithering: {}\n", enable_dither);

    let wait_time = Duration::from_millis(mouse_wait);
    let enigo = Enigo::new();
    let mut easel = Easel::new(easel_config, enigo, wait_time)?;
    let mut image = size_to_easel(&image::open(image_path)?, &easel).to_rgba();
    let palette = Palette::new();
    if enable_dither {
        dither(&mut image, &palette);
    }

    easel.change_brush_size(0);

    let (size_x, size_y) = image.dimensions();
    if (size_x > size_y && easel.orientation == Orientation::Portrait)
        || (size_y > size_x && easel.orientation == Orientation::Landscape)
    {
        easel.change_orientation();
    }

    let (ulcorner, lrcorner) = easel.get_bounds();
    let easel_y = lrcorner.1 - ulcorner.1 - 1;
    let easel_x = lrcorner.0 - ulcorner.0 - 1;

    let size_x = size_x as i32;
    let size_y = size_y as i32;
    let mut current_color = easel.current_color;
    let mut start_x = 0_i32;
    let mut start_y = 0_i32;
    let mut paused = false;
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let x = x as i32;
        let y = y as i32;
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

        let rgba = pixel.to_rgba();
        let closest_color = palette.colormap[palette.index_of(&rgba)];

        // If we've hit the end of a row, draw the rest of the row before
        // moving on to the next row.
        if y > start_y {
            easel.draw_line(
                (start_x, start_y),
                (size_x, start_y),
                &current_color,
            )?;
            if size_x < easel_x {
                easel.draw_line(
                    (size_x + 1, start_y),
                    (easel_x, start_y),
                    &PaletteColor::White,
                )?;
                easel.change_color(&closest_color);
            }
            start_x = x;
            start_y = y;
            current_color = closest_color;
        }

        // If there's a color change, draw the line up to this pixel and stop.
        if closest_color != current_color {
            easel.draw_line(
                (start_x, start_y),
                (x - 1, y),
                &current_color,
            )?;
            start_x = x;
            start_y = y;
            current_color = closest_color;
        }
    }

    // Clean up the right-most edge of the picture if one exists.
    if size_x < easel_x {
        easel.draw_line(
            (size_x, 0),
            (size_x, size_y),
            &PaletteColor::White,
        )?;
    }

    // Once we've hit the end of the picture, tidy up the bottom by drawing
    // white lines to fill in the entire canvas.
    if size_y < easel_y {
        for iy in size_y..easel_y {
            easel.draw_line(
                (0, iy),
                (easel_x, iy),
                &PaletteColor::White,
            )?;
        }
    }

    Ok(())
}

fn main() {
    app().unwrap();
}
