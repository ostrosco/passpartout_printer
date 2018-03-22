extern crate enigo;
extern crate failure;
extern crate passpartout_printer;

use std::time::Duration;
use enigo::Enigo;
use passpartout_printer::easel::Easel;
use passpartout_printer::colors::PaletteColor;
use failure::Error;

fn app() -> Result<(), Error> {
    let enigo = Enigo::new();
    let mut easel = Easel::new(
        "../coords.json".to_string(),
        enigo,
        Duration::from_millis(6),
    )?;
    easel.change_brush_size(0);
    let points = [(100, 100), (100, 150), (150, 150), (150, 100)];
    easel.draw_shape(&points, &PaletteColor::Red, true, true)?;
    let points = [(125, 50), (100, 100), (150, 100)];
    easel.draw_shape(&points, &PaletteColor::Blue, true, true)?;
    let points = [
        (200, 200),
        (150, 250),
        (100, 250),
        (150, 300),
        (125, 350),
        (200, 300),
        (250, 350),
        (225, 300),
        (300, 250),
        (250, 250),
    ];
    easel.draw_shape(&points, &PaletteColor::Yellow, true, true)?;
    Ok(())
}

fn main() {
    app().unwrap();
}
