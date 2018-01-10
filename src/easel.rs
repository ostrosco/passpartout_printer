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
        enigo.mouse_move_to(orient_coords.0, orient_coords.1);
        thread::sleep(*wait_time);
        enigo.mouse_down(MouseButton::Left);
        thread::sleep(*wait_time);
        enigo.mouse_up(MouseButton::Left);
        thread::sleep(*wait_time);
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
}
