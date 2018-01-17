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

#[derive(Debug)]
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

    fn get_hex(&self) -> i32 {
        match *self {
            Color::Black => 0x0d_0d_0d,
            Color::Grey => 0x76_76_76,
            Color::White => 0xe5_e5_e5,
            Color::DarkBrown => 0x62_32_00,
            Color::Brown => 0xb9_7a_56,
            Color::LightBrown => 0xef_e4_b0,
            Color::DarkRed => 0x7e_0d_0d,
            Color::Red => 0xed_1c_22,
            Color::Pink => 0xff_ae_c9,
            Color::Orange => 0xff_7f_26,
            Color::DarkYellow => 0xff_c9_0d,
            Color::Yellow => 0xfa_ed_16,
            Color::DarkGreen => 0x26_5d_38,
            Color::Green => 0x35_ab_55,
            Color::LightGreen => 0xb5_e6_1c,
            Color::DarkBlue => 0x00_65_91,
            Color::Blue => 0x00_a2_e8,
            Color::LightBlue => 0x99_d9_ea,
            Color::DarkIndigo => 0x1c_22_63,
            Color::Indigo => 0x30_39_cc,
            Color::LightIndigo => 0x70_92_be,
            Color::DarkViolet => 0x95_35_96,
            Color::Violet => 0xd5_5f_d7,
            Color::LightViolet => 0xc1_a7_d7,
        }
    }

    fn find_closest_color(color: i32) -> Color {
        let colors = vec![
            Color::Black,
            Color::Grey,
            Color::White,
            Color::DarkBrown,
            Color::Brown,
            Color::LightBrown,
            Color::DarkRed,
            Color::Red,
            Color::Pink,
            Color::Orange,
            Color::DarkYellow,
            Color::Yellow,
            Color::DarkGreen,
            Color::Green,
            Color::LightGreen,
            Color::DarkBlue,
            Color::Blue,
            Color::LightBlue,
            Color::DarkIndigo,
            Color::Indigo,
            Color::LightIndigo,
            Color::DarkViolet,
            Color::Violet,
            Color::LightViolet,
        ];
        let (r, g, b) = (color >> 16 & 0xff, color >> 8 & 0xff, color & 0xff);
        let mut closest = Color::Black;
        let mut color_dist = 0xff_ff_ff;

        // Iterate over all colors and compare the RBG values to find the
        // closest value to the input color.
        for col in colors {
            let hex = col.get_hex();
            let (col_r, col_g, col_b) =
                (hex >> 16 & 0xff, hex >> 8 & 0xff, hex & 0xff);
            let curr_color_dist =
                (col_r - r).abs() + (col_g - g).abs() + (col_b - b).abs();
            if curr_color_dist < color_dist {
                closest = col;
                color_dist = curr_color_dist;
            }
        }
        closest
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

#[derive(Fail, Debug)]
pub enum EaselError {
    #[fail(display = "Out of bounds error drawing to the easel")]
    OutOfBounds,
}

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
        let brush_size = brush_size.max(0).min(NUM_BRUSH_STEPS);
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

    pub fn draw_pixel(
        &self,
        coords: Point,
        color: i32,
        enigo: &mut Enigo,
        wait_time: &Duration,
    ) -> Result<(), Error> {
        let closest_color = Color::find_closest_color(color);

        // Translate the coordinates of the picture to coordinates of the easel.
        let (start, end) = self.get_bounds();
        let coords = (start.0 + coords.0, start.1 + coords.1);
        if coords.0 > end.0 || coords.1 > end.1 {
            Err(EaselError::OutOfBounds)?
        }

        self.change_color(&closest_color, enigo, wait_time);
        move_and_click(coords.0, coords.1, enigo, wait_time);
        Ok(())
    }
}
