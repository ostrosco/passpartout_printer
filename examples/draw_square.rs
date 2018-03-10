extern crate passpartout_printer;
extern crate enigo;
extern crate failure;

use std::time::Duration;
use enigo::Enigo;
use passpartout_printer::easel::Easel;
use passpartout_printer::colors::PaletteColor;
use failure::Error;

fn app() -> Result<(), Error> {
    let enigo = Enigo::new();
    let mut easel = Easel::new("../coords.json".to_string(), enigo, Duration::from_millis(6))?;
    easel.change_brush_size(0);
    let points = vec![(100, 100), (100, 150), (150, 150), (150, 100)];
    easel.draw_shape(points, &PaletteColor::Red, false)
}

fn main() { 
    app();
}
