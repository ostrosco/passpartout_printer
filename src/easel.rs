extern crate failure;

extern crate enigo;
extern crate serde;
extern crate serde_json;

use std::io::{Read, Write};
use std::fs::File;
use std::time::Duration;
use std::thread;
use self::failure::Error;
use self::enigo::{Enigo, MouseButton, MouseControllable};
use colors::{Palette, PaletteColor, Point};
use image::imageops::ColorMap;
use image::Rgba;

#[derive(Serialize, Deserialize)]
pub struct EaselCoords {
    pub portrait_bounds: (Point, Point),
    pub landscape_bounds: (Point, Point),
    pub paintbrush: Point,
    pub spray_can: Point,
    pub pen: Point,
    pub decrease_brush: Point,
    pub increase_brush: Point,
    pub change_orientation: Point,
    pub color_start: Point,
    pub color_row_step: i32,
    pub color_col_step: i32,
}

impl EaselCoords {
    pub fn new(path: String) -> Result<EaselCoords, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let win: EaselCoords = serde_json::from_str(&contents)?;
        Ok(win)
    }

    pub fn save(&self, path: &str) -> Result<(), Error> {
        let j = serde_json::to_string_pretty(self)?;
        let mut file = File::create(&path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
    }
}

#[derive(PartialEq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

const NUM_BRUSH_STEPS: i32 = 16;
const STARTING_BRUSH: i32 = 9;
const STARTING_COLOR: PaletteColor = PaletteColor::Black;

#[derive(Fail, Debug)]
pub enum EaselError {
    #[fail(display = "Out of bounds error drawing to the easel")] OutOfBounds,
}

pub struct Easel {
    pub mouse: Enigo,
    pub mouse_wait: Duration,
    pub easel_coords: EaselCoords,
    pub orientation: Orientation,
    pub brush_size: i32,
    pub current_color: PaletteColor,
}

impl Easel {
    pub fn new(path: String, mouse: Enigo, mouse_wait: Duration) -> Result<Easel, Error> {
        let easel_coords = EaselCoords::new(path)?;
        let orientation = Orientation::Portrait;
        Ok(Easel {
            mouse: mouse,
            mouse_wait: mouse_wait,
            easel_coords: easel_coords,
            orientation: orientation,
            brush_size: STARTING_BRUSH,
            current_color: STARTING_COLOR,
        })
    }

    fn click(&mut self, wait_time: Option<Duration>) {
        let mouse_wait = match wait_time {
            Some(w) => w,
            None => self.mouse_wait,
        };
        self.mouse.mouse_down(MouseButton::Left);
        thread::sleep(mouse_wait);
        self.mouse.mouse_up(MouseButton::Left);
        thread::sleep(mouse_wait);
    }

    fn move_and_click(&mut self, x: i32, y: i32) {
        self.mouse.mouse_move_to(x, y);
        thread::sleep(self.mouse_wait);
        self.click(None);
    }

    pub fn change_orientation(
        &mut self,
    ) {
        let orient_coords = self.easel_coords.change_orientation;
        self.move_and_click(orient_coords.0, orient_coords.1);
        self.orientation = match self.orientation {
            Orientation::Portrait => Orientation::Landscape,
            Orientation::Landscape => Orientation::Portrait,
        };
    }

    pub fn get_bounds(&self) -> (Point, Point) {
        match self.orientation {
            Orientation::Portrait => self.easel_coords.portrait_bounds,
            Orientation::Landscape => self.easel_coords.landscape_bounds,
        }
    }

    pub fn change_color(
        &mut self,
        color: &PaletteColor,
    ) {
        if *color != self.current_color {
            let (row, col) = color.get_row_col();
            let row_step = self.easel_coords.color_row_step;
            let col_step = self.easel_coords.color_col_step;
            let (x, y) = (
                self.easel_coords.color_start.0 + (row * row_step),
                self.easel_coords.color_start.1 + (col * col_step),
            );
            self.move_and_click(x, y);
            self.current_color = *color;
        }
    }

    pub fn change_brush_size(
        &mut self,
        brush_size: i32,
    ) {
        // For some reason, changing the brush size is very inconsistent
        // at speeds faster than 32 ms, so we slow down here only for
        // brush change sizes.
        let mouse_wait = Duration::from_millis(32);
        
        // Make sure that we aren't going to accidentally set an internal
        // brush size greater or smaller than what the game supports.
        let brush_size = brush_size.max(0).min(NUM_BRUSH_STEPS);
        let brush_coords = if brush_size > self.brush_size {
            self.easel_coords.increase_brush
        } else {
            self.easel_coords.decrease_brush
        };
        let num_clicks = (brush_size - self.brush_size).abs();
        self.move_and_click(brush_coords.0, brush_coords.1);
        for _ in 1..num_clicks {
            self.click(Some(mouse_wait));
            thread::sleep(mouse_wait);
        }
        self.brush_size = brush_size;
    }

    pub fn draw_pixel(
        &mut self,
        coords: Point,
        color: &Rgba<u8>,
    ) -> Result<(), Error> {
        let palette = Palette::new();
        let closest_color = palette.colormap[palette.index_of(color)];

        // Translate the coordinates of the picture to coordinates of the easel.
        let (start, end) = self.get_bounds();
        let brush_size = self.brush_size + 12;
        let coords = (
            start.0 + coords.0 + brush_size,
            start.1 + coords.1 + brush_size,
        );
        if coords.0 > end.0 || coords.1 > end.1 {
            Err(EaselError::OutOfBounds)?
        }

        self.change_color(&closest_color);
        self.move_and_click(coords.0, coords.1);
        Ok(())
    }

    pub fn draw_line(
        &mut self,
        start_line: Point,
        end_line: Point,
        color: &PaletteColor,
    ) -> Result<(), Error> {
        // Translate the coordinates of the picture to coordinates of the easel.
        let (start, end) = self.get_bounds();
        let start_draw = ((start.0 + start_line.0), (start.1 + start_line.1));
        if start_draw.0 > end.0 || start_draw.1 > end.1 {
            Err(EaselError::OutOfBounds)?
        }

        let end_draw = ((start.0 + end_line.0), (start.1 + end_line.1));
        if end_draw.0 > end.0 || end_draw.1 > end.1 {
            Err(EaselError::OutOfBounds)?
        }

        self.change_color(color);
        self.mouse.mouse_move_to(start_draw.0, start_draw.1);
        thread::sleep(self.mouse_wait);
        self.mouse.mouse_down(MouseButton::Left);
        thread::sleep(self.mouse_wait);
        self.mouse.mouse_move_to(end_draw.0, end_draw.1);
        thread::sleep(self.mouse_wait);
        self.mouse.mouse_up(MouseButton::Left);
        thread::sleep(self.mouse_wait);
        Ok(())
    }
}
