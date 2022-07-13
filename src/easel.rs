use crate::colors::PaletteColor;
use crate::coords::Coord;
use enigo::{Enigo, MouseButton, MouseControllable};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::f32;
use std::fs::File;
use std::i32;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
/// A structure containing the location of all the tools on the easel.
pub struct EaselCoords {
    pub portrait_bounds: (Coord, Coord),
    pub landscape_bounds: (Coord, Coord),
    pub paintbrush: Coord,
    pub spray_can: Coord,
    pub pen: Coord,
    pub decrease_brush: Coord,
    pub increase_brush: Coord,
    pub change_orientation: Coord,
    pub color_start: Coord,
    pub color_row_step: i32,
    pub color_col_step: i32,
}

impl EaselCoords {
    pub fn new(path: String) -> Result<EaselCoords, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let win: EaselCoords = serde_json::from_str(&contents)?;
        Ok(win)
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let j = serde_json::to_string_pretty(self)?;
        let mut file = File::create(&path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
    }
}

#[derive(PartialEq)]
/// An enumeration that describes the two orientations the canvas can be.
pub enum Orientation {
    Portrait,
    Landscape,
}

// Though we don't currently use the Pen or Spraycan options, we're leaving them in the enumeration
// for completeness.
#[allow(dead_code)]

/// A list of the tools that Passpartout provides for drawing.
#[derive(PartialEq)]
pub enum Tool {
    Paintbrush,
    Pen,
    Spraycan,
}

/// The number of brush steps we can take when resizing.
const NUM_BRUSH_STEPS: i32 = 16;

/// From a fresh boot of the game, the brush color starts as black.
const STARTING_COLOR: PaletteColor = PaletteColor::Black;

/// From a fresh boot of the game, the paintbrush is the active tool.
const STARTING_TOOL: Tool = Tool::Paintbrush;

#[derive(Debug)]
/// A list of potential errors that can occur while drawing to the easel.
pub enum EaselError {
    OutOfBounds,
    NoCoord,
}

impl std::fmt::Display for EaselError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EaselError::OutOfBounds => write!(f, "out of bounds"),
            EaselError::NoCoord => write!(f, "invalid coordinates"),
        }
    }
}

impl Error for EaselError {}

/// A structure keeping track of the current state of the easel and provides means to change
/// the state of the easel via mouse clicks.
pub struct Easel {
    /// The object for manipulating the mouse.
    pub mouse: Enigo,

    /// The amount of time to wait between mouse moves and clicks.
    pub mouse_wait: Duration,

    /// A mapping of where elements of the easel are in screen coordinates.
    pub easel_coords: EaselCoords,

    /// The current orientation of the easel: landscape or portrait.
    pub orientation: Orientation,

    /// The current brush size.
    pub brush_size: i32,

    /// The active color of the brush.
    pub current_color: PaletteColor,

    /// The active tool for drawing.
    pub current_tool: Tool,
}

impl Easel {
    /// Create a new easel. When creating a new easel, it's important to set the mouse wait
    /// properly. If you start seeing lines being drawn from the easel towards the color palette,
    /// you likely need to increase the mouse wait time.
    ///
    /// Initial experimentation has shown that, regardless of FPS, anything lower than 6 ms causes
    /// the game to not recognize that the mouse button has been released and will make many, many
    /// mistakes in drawing.
    ///
    /// # Arguments
    ///
    /// * `path`: Path to the JSON file containing the coordinates of easel elements in-game.
    /// * `mouse`: An Enigo structure used to manipulate the mouse position.
    /// * `mouse_wait`: The time to wait between mouse operations.
    ///
    pub fn new(path: String, mouse: Enigo, mouse_wait: Duration) -> Result<Easel, Box<dyn Error>> {
        let easel_coords = EaselCoords::new(path)?;
        let orientation = Orientation::Portrait;

        let mut easel = Easel {
            mouse,
            mouse_wait,
            easel_coords,
            orientation,
            brush_size: 0,
            current_color: STARTING_COLOR,
            current_tool: STARTING_TOOL,
        };

        // Since this application could be run multiple times in succession,
        // this initial state isn't always true. However, we can guarantee
        // that it is for the current color, current tool, and brush size.
        // We can't unfortunately do anything clever to guarantee the
        // easel orientation, so that'll have to be on the user to make sure
        // they switch back to portrait between runs.
        println!("Resetting easel to known configuration.");
        println!("Assuming a default orientation of portrait.");
        easel.change_brush_size(16);
        easel.change_brush_size(0);
        easel.change_tool(STARTING_TOOL);
        easel.change_color(&STARTING_COLOR);
        println!("Easel configuration complete.");

        Ok(easel)
    }

    fn click_custom_wait(&mut self, wait: Duration) {
        self.mouse.mouse_down(MouseButton::Left);
        thread::sleep(wait);
        self.mouse.mouse_up(MouseButton::Left);
        thread::sleep(wait);
    }

    fn click(&mut self) {
        self.click_custom_wait(self.mouse_wait);
    }

    fn move_and_click(&mut self, coord: &Coord) {
        self.mouse.mouse_move_to(coord.x, coord.y);
        self.click();
    }

    /// Toggles the orientation of the easel.
    pub fn change_orientation(&mut self) {
        let orient_coords = self.easel_coords.change_orientation;
        self.move_and_click(&orient_coords);
        self.orientation = match self.orientation {
            Orientation::Portrait => Orientation::Landscape,
            Orientation::Landscape => Orientation::Portrait,
        };
    }

    /// Changes the current tool.
    pub fn change_tool(&mut self, tool: Tool) {
        let coords = match tool {
            Tool::Paintbrush => self.easel_coords.paintbrush,
            Tool::Pen => self.easel_coords.pen,
            Tool::Spraycan => self.easel_coords.spray_can,
        };
        self.move_and_click(&coords);
        self.current_tool = tool;
    }

    /// Returns the current bounds of the easel in screen coordinates.
    pub fn get_bounds(&self) -> (Coord, Coord) {
        match self.orientation {
            Orientation::Portrait => self.easel_coords.portrait_bounds,
            Orientation::Landscape => self.easel_coords.landscape_bounds,
        }
    }

    /// Changes from the current color to the desired color. Does nothing if the current color is
    /// the same as the desired color.
    pub fn change_color(&mut self, color: &PaletteColor) {
        if *color != self.current_color {
            let color_pos = color.get_row_col();
            let row_step = self.easel_coords.color_row_step;
            let col_step = self.easel_coords.color_col_step;
            let color_coords = Coord::new(
                self.easel_coords.color_start.x + (color_pos.x * row_step),
                self.easel_coords.color_start.y + (color_pos.y * col_step),
            );
            self.move_and_click(&color_coords);
            self.current_color = *color;
        }
    }

    /// Changes the brush size in fixed steps by clicking on the brushes to increase or decrease
    /// the size.
    ///
    /// # Arguments
    ///
    /// * `brush_size` - The size from 0 to `NUM_BRUSH_STEPS` to change the brush size to.
    ///
    /// # Example
    /// ```no_run
    /// use enigo::*;
    /// use passpartout_printer::easel::Easel;
    /// use std::time::Duration;
    /// use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let mut easel = Easel::new("coords.json".into(), Enigo::new(), Duration::from_millis(6))?;
    /// easel.change_brush_size(0); // shrinks the brush to its minimum size
    /// # Ok(())
    /// # }
    /// ```
    pub fn change_brush_size(&mut self, brush_size: i32) {
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
        self.move_and_click(&brush_coords);
        for _ in 1..num_clicks {
            self.click_custom_wait(mouse_wait);
        }
        self.brush_size = brush_size;
    }

    /// Draws a line on the easel of the particular color.
    ///
    /// # Arguments
    ///
    /// * `start_line`: The starting point of the line in image coordinates.
    /// * `end_line`: The end point of the line in image coordinates.
    /// * `color`: The color of the line.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use passpartout_printer::{
    ///     easel::Easel,
    ///     coords::Coord,
    ///     colors::PaletteColor,
    /// };
    /// use enigo::*;
    /// use std::time::Duration;
    /// use image::Rgba;
    /// use std::error::Error;
    ///
    /// // Draw a black diagonal line from the upper-left corner
    /// // 100 pixels away.
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let mut easel = Easel::new("coords.json".into(), Enigo::new(), Duration::from_millis(6))?;
    /// let color = PaletteColor::Black;
    /// let start = Coord::new(0, 0);
    /// let end = Coord::new(100, 100);
    /// easel.draw_line(start, end, &color);
    /// # Ok(())
    /// # }
    /// ```
    pub fn draw_line(
        &mut self,
        start_line: Coord,
        end_line: Coord,
        color: &PaletteColor,
    ) -> Result<(), Box<dyn Error>> {
        // Translate the coordinates of the picture to coordinates of the easel.
        self.draw_shape(&[start_line, end_line], color, false, false)
    }

    /// Draws an arbitrary shape to the easel. This draws the shape as one
    /// continuous stroke.
    ///
    /// # Arguments
    ///
    /// * `points`: The points defining the polygon.
    /// * `color`: The color of the shape.
    /// * `close_shape`: Whether or not to connect the first and last points.
    /// * `fill`: Whether or not to fill the shape. Implies close_shape.
    ///
    pub fn draw_shape(
        &mut self,
        points: &[Coord],
        color: &PaletteColor,
        close_shape: bool,
        fill: bool,
    ) -> Result<(), Box<dyn Error>> {
        let (start, end) = self.get_bounds();
        self.change_color(color);

        let start_point = match points.get(0) {
            Some(p) => p,
            None => Err(EaselError::NoCoord)?,
        };

        let start_point = start + start_point;
        if start_point.x > end.x || start_point.y > end.y {
            println!("point: {:?}, end: {:?}", start_point, end);
            Err(EaselError::OutOfBounds)?
        }

        self.mouse.mouse_move_to(start_point.x, start_point.y);
        thread::sleep(self.mouse_wait);
        self.mouse.mouse_down(MouseButton::Left);
        for point in points.iter() {
            let point = start + point;
            if point.x > end.x || point.y > end.y {
                println!("point: {:?}, end: {:?}", point, end);
                Err(EaselError::OutOfBounds)?
            }
            self.mouse.mouse_move_to(point.x, point.y);
            thread::sleep(self.mouse_wait);
        }

        if close_shape || fill {
            self.mouse.mouse_move_to(start_point.x, start_point.y);
            thread::sleep(self.mouse_wait);
        }

        self.mouse.mouse_up(MouseButton::Left);
        thread::sleep(self.mouse_wait);

        if fill {
            self.fill(points, color)?;
        }

        Ok(())
    }

    /// Use the scanline polygon fill algorithm to fill in the polygon.
    ///
    /// * `points` List of coordinates that define the polygon to fill.
    /// * `color` The color to fill the polygon with.
    ///
    fn fill(&mut self, points: &[Coord], color: &PaletteColor) -> Result<(), Box<dyn Error>> {
        let mut edges: Vec<[&Coord; 2]> = points.windows(2).map(|pts| [&pts[0], &pts[1]]).collect();
        edges.push([points.first().unwrap(), points.last().unwrap()]);
        let slope: Vec<f32> = edges
            .iter()
            .map(|pts| {
                if pts[1].x == pts[0].x {
                    0.0
                } else {
                    (pts[1].y - pts[0].y) as f32 / (pts[1].x - pts[0].x) as f32
                }
            })
            .collect();
        let start_y = points.iter().fold(i32::MAX, |acc, pnt| acc.min(pnt.y));
        let end_y = points.iter().fold(0, |acc, pnt| acc.max(pnt.y));
        let mut iy = start_y;
        let old_brush_size = self.brush_size;
        self.change_brush_size(0);

        while iy < end_y {
            let mut active_edges = vec![];
            for (edge, slope) in edges.iter().zip(slope.iter()) {
                let max = edge[0].y.max(edge[1].y);
                let min = edge[0].y.min(edge[1].y);
                if max > iy && min < iy && max != min {
                    active_edges.push((edge, slope));
                }
            }
            let x_draw: Vec<i32> = active_edges
                .iter()
                .map(|&(pts, m)| {
                    if *m == 0.0 {
                        pts[0].x
                    } else {
                        (pts[0].x as f32 + 1.0 / m * (iy - pts[0].y) as f32) as i32
                    }
                })
                .collect();
            let mut in_poly = true;
            for x in x_draw.windows(2) {
                if x[0] != x[1] && in_poly {
                    self.draw_line(Coord::new(x[0], iy), Coord::new(x[1], iy), color)?;
                }
                in_poly = !in_poly;
            }

            // Since the brush size is 0, we increment by half of the brush
            // size, or 6 pixels.
            iy += 6;
        }

        self.change_brush_size(old_brush_size);
        Ok(())
    }
}
