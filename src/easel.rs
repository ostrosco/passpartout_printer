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

pub type Point = (i32, i32);

pub enum Color {
    Black,
    Grey,
    White,
    DarkBrown,
    Brown,
    LightBrown,
    DarkRed,
    Red,
    Pink,
    Orange,
    DarkYellow,
    Yellow,
    DarkGreen,
    Green,
    LightGreen,
    DarkBlue,
    Blue,
    LightBlue,
    DarkIndigo,
    Indigo,
    LightIndigo,
    DarkViolet,
    Violet,
    LightViolet,
}

impl Color {
    fn get_row_col(&self) -> Point {
        match *self {
            Color::Black => (0, 0),
            Color::Grey => (0, 1),
            Color::White => (0, 2),
            Color::DarkBrown => (1, 0),
            Color::Brown => (1, 1),
            Color::LightBrown => (1, 2),
            Color::DarkRed => (2, 0),
            Color::Red => (2, 1),
            Color::Pink => (2, 2),
            Color::Orange => (3, 0),
            Color::DarkYellow => (3, 1),
            Color::Yellow => (3, 2),
            Color::DarkGreen => (4, 0),
            Color::Green => (4, 1),
            Color::LightGreen => (4, 2),
            Color::DarkBlue => (5, 0),
            Color::Blue => (5, 1),
            Color::LightBlue => (5, 2),
            Color::DarkIndigo => (6, 0),
            Color::Indigo => (6, 1),
            Color::LightIndigo => (6, 2),
            Color::DarkViolet => (7, 0),
            Color::Violet => (7, 1),
            Color::LightViolet => (7, 2),
        }
    }
}

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

pub enum Orientation {
    Portrait,
    Landscape,
}

const NUM_BRUSH_STEPS: i32 = 16;
const STARTING_BRUSH: i32 = 9;

pub struct Easel {
    pub easel_coords: EaselCoords,
    pub orientation: Orientation,
    pub brush_size: i32,
}

fn click(enigo: &mut Enigo, wait_time: &Duration) {
    enigo.mouse_down(MouseButton::Left);
    thread::sleep(*wait_time);
    enigo.mouse_up(MouseButton::Left);
    thread::sleep(*wait_time);
}

fn move_and_click(x: i32, y: i32, enigo: &mut Enigo, wait_time: &Duration) {
    enigo.mouse_move_to(x, y);
    thread::sleep(*wait_time);
    click(enigo, wait_time);
}

impl Easel {
    pub fn new(path: String) -> Result<Easel, Error> {
        let easel_coords = EaselCoords::new(path)?;
        let orientation = Orientation::Portrait;
        Ok(Easel {
            easel_coords: easel_coords,
            orientation: orientation,
            brush_size: STARTING_BRUSH,
        })
    }

    pub fn change_orientation(
        &mut self,
        enigo: &mut Enigo,
        wait_time: &Duration,
    ) {
        let orient_coords = self.easel_coords.change_orientation;
        move_and_click(orient_coords.0, orient_coords.1, enigo, wait_time);
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
        &self,
        color: &Color,
        enigo: &mut Enigo,
        wait_time: &Duration,
    ) {
        let (row, col) = color.get_row_col();
        let row_step = self.easel_coords.color_row_step;
        let col_step = self.easel_coords.color_col_step;
        let (x, y) = (
            self.easel_coords.color_start.0 + (row * row_step),
            self.easel_coords.color_start.1 + (col * col_step),
        );
        move_and_click(x, y, enigo, wait_time);
    }

    pub fn change_brush_size(
        &mut self,
        brush_size: i32,
        enigo: &mut Enigo,
        wait_time: &Duration,
    ) {
        // Make sure that we aren't going to accidentally set an internal
        // brush size greater or smaller than what the game supports.
        let brush_size = brush_size.max(1).min(NUM_BRUSH_STEPS);
        let brush_coords = if brush_size > self.brush_size {
            self.easel_coords.increase_brush
        } else {
            self.easel_coords.decrease_brush
        };
        let num_clicks = (brush_size - self.brush_size).abs();
        move_and_click(brush_coords.0, brush_coords.1, enigo, wait_time);
        for _ in 1..num_clicks {
            click(enigo, wait_time);
            thread::sleep(*wait_time);
        }
        self.brush_size = brush_size;
    }
}
