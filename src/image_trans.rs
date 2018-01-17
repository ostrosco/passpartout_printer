extern crate enigo;
extern crate failure;
extern crate image;

use self::failure::Error;
use easel::{Color, Easel};
use enigo::Enigo;
use std::time::Duration;
use self::image::Pixel;

pub fn draw_image(
    image_path: &str,
    easel_config: String,
    enigo: &mut Enigo,
    wait_time: &Duration,
) -> Result<(), Error> {
    let mut easel = Easel::new(easel_config)?;
    let mut image = image::open(image_path)?.to_rgb();
    easel.change_brush_size(0, enigo, wait_time);
    easel.change_orientation(enigo, wait_time);

    // Experiment!
    let mut current_color = easel.current_color;
    let mut start_x = 0;
    let mut start_y = 0;
    let (size_x, _) = image.dimensions();
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let rgb = pixel.to_rgb().data;
        let rgb = (rgb[0], rgb[1], rgb[2]);
        let closest_color = Color::find_closest_color(rgb);

        // If we've hit the end of a row, draw the previous row.
        if y > start_y {
            easel.draw_line(
                (start_x as i32, start_y as i32),
                (size_x as i32, start_y as i32),
                &current_color,
                enigo,
                wait_time,
            )?;
            start_x = x;
            start_y = y;
            current_color = closest_color;
        }
        if closest_color != current_color {
            easel.draw_line(
                (start_x as i32, start_y as i32),
                (x as i32, y as i32),
                &current_color,
                enigo,
                wait_time,
            )?;
            println!("new start: {} {}", x, y);
            start_x = x;
            start_y = y;
            current_color = closest_color;
        }
    }
    Ok(())
}
