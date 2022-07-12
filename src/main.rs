#[macro_use]
extern crate clap;

use clap::App;
use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::Enigo;
use image::imageops::dither;
use image::Pixel;
use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::u64;

mod colors;
mod coords;
mod easel;
mod image_drawer;
mod manual_config;

use colors::Palette;
use easel::Easel;
use image_drawer::size_to_easel;
use image_drawer::ImageDrawer;

fn app() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Passpartout Printer")
        .version("0.1.0")
        .args_from_usage(
            "-w, --mouse-wait=[WAIT] 'Specify the time to wait between mouse actions'
            --configure 'Configures the application with coordinates in-game.'
            --enable-dither 'Enables dithering to reduce color banding but increase draw time'
            --no-scale 'Disable scaling of the input image.'
            -i, --image=[IMAGE] 'Input image to use'",
        )
        .get_matches();

    if matches.occurrences_of("configure") > 0 {
        return manual_config::create_config("coords.json");
    }

    let (tx, rx) = mpsc::channel();

    // A simple event loop to search for the escape key to pause drawing.
    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut prev = false;
        loop {
            let key_pressed = device_state.get_keys();
            if key_pressed.contains(&Keycode::Space) && key_pressed.contains(&Keycode::LControl) {
                prev = true;
            } else if prev {
                prev = false;
                tx.send(()).unwrap();
            }
        }
    });

    let easel_config = String::from("coords.json");
    let image_path: String = matches
        .value_of("image")
        .expect("Please enter a path to the image to draw.")
        .to_string();
    let mouse_wait: u64 = value_t!(matches, "mouse-wait", u64).unwrap_or(7);
    let enable_dither: bool = !matches!(matches.occurrences_of("enable-dither"), 0);
    let enable_scale: bool = matches!(matches.occurrences_of("no-scale"), 0);

    println!("Printing to Passpartout with the following settings:");
    println!("-- image: {}", image_path);
    println!("-- mouse wait: {}", mouse_wait);
    println!("-- dithering: {}", enable_dither);
    println!("-- image scaling: {}", enable_scale);
    println!();
    println!("Press Left Control + Space to pause drawing.");
    println!();

    let wait_time = Duration::from_millis(mouse_wait);
    let enigo = Enigo::new();
    let mut easel = Easel::new(easel_config, enigo, wait_time)?;
    let mut image = if enable_scale {
        size_to_easel(&image::open(image_path)?, &easel).into_rgba8()
    } else {
        image::open(image_path)?.into_rgba8()
    };
    let palette = Palette::new();
    if enable_dither {
        dither(&mut image, &palette);
    }

    let (size_x, size_y) = image.dimensions();
    let mut image_drawer = ImageDrawer::new(&mut easel, size_x, size_y);
    image_drawer.draw_top_border()?;

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

        let mut rgba = pixel.to_rgba();
        image_drawer.handle_pixel(&mut rgba, x, y)?;
    }

    image_drawer.cleanup_image()?;

    Ok(())
}

fn main() {
    app().unwrap();
}
