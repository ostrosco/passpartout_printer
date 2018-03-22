extern crate failure;

extern crate enigo;
extern crate serde;
extern crate serde_json;

use std::io::{Read, Write};
use std::fs::File;
use std::f32;
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

#[derive(PartialEq)]
pub enum Tool {
    Paintbrush,
    Pen,
    Spraycan,
}

/// The number of brush steps we can take when resizing.
const NUM_BRUSH_STEPS: i32 = 16;

/// From a fresh boot of the game, the brush size starts at step 9.
const STARTING_BRUSH: i32 = 9;

/// From a fresh boot of the game, the brush color starts as black.
const STARTING_COLOR: PaletteColor = PaletteColor::Black;

/// From a fresh boot of the game, the paintbrush is the active tool.
const STARTING_TOOL: Tool = Tool::Paintbrush;

#[derive(Fail, Debug)]
pub enum EaselError {
    #[fail(display = "Out of bounds error drawing to the easel")] OutOfBounds,
    #[fail(display = "Drawing requires at least one point.")] NoPoints,
}

pub struct Easel {
    /// The enigo object for manipulating the mouse.
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
    /// Create a new easel. When creating a new easel, it's important to set
    /// the mouse wait properly. If you start seeing lines being drawn from
    /// the easel towards the color palette, you likely need to increase the
    /// mouse wait time.
    ///
    /// Initial experimentation has shown that, regardless of
    /// FPS, anything lower than 6 ms causes the game to not recognize that
    /// the mouse button has been released and will make many, many mistakes
    /// in drawing.
    ///
    /// # Arguments
    ///
    /// * `path`: Path to the JSON file containing the coordinates of easel
    ///           elements in-game.
    /// * `mouse`: An Enigo structure used to manipulate the mouse position.
    /// * `mouse_wait`: The time to wait between mouse operations.
    ///
    pub fn new(
        path: String,
        mouse: Enigo,
        mouse_wait: Duration,
    ) -> Result<Easel, Error> {
        let easel_coords = EaselCoords::new(path)?;
        let orientation = Orientation::Portrait;
        Ok(Easel {
            mouse: mouse,
            mouse_wait: mouse_wait,
            easel_coords: easel_coords,
            orientation: orientation,
            brush_size: STARTING_BRUSH,
            current_color: STARTING_COLOR,
            current_tool: STARTING_TOOL,
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

    /// Toggles the orientation of the easel.
    pub fn change_orientation(&mut self) {
        let orient_coords = self.easel_coords.change_orientation;
        self.move_and_click(orient_coords.0, orient_coords.1);
        self.orientation = match self.orientation {
            Orientation::Portrait => Orientation::Landscape,
            Orientation::Landscape => Orientation::Portrait,
        };
    }

    /// Changes the current tool.
    pub fn change_tool(&mut self, tool: Tool) {
        if tool == self.current_tool {
            return;
        }
        let coords = match tool {
            Tool::Paintbrush => self.easel_coords.paintbrush,
            Tool::Pen => self.easel_coords.pen,
            Tool::Spraycan => self.easel_coords.spray_can,
        };
        self.move_and_click(coords.0, coords.1);
        self.current_tool = tool;
    }

    /// Returns the current bounds of the easel in screen coordinates.
    pub fn get_bounds(&self) -> (Point, Point) {
        match self.orientation {
            Orientation::Portrait => self.easel_coords.portrait_bounds,
            Orientation::Landscape => self.easel_coords.landscape_bounds,
        }
    }

    /// Changes from the current color to the desired color. Does nothing
    /// if the current color is the same as the desired color.
    pub fn change_color(&mut self, color: &PaletteColor) {
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

    /// Changes the brush size in fixed steps by clicking on the brushes
    /// to increase or decrease the size.
    ///
    /// # Arguments
    ///
    /// * `brush_size` - The size from 0 to `NUM_BRUSH_STEPS` to change the
    ///    brush size to.
    ///
    /// # Example
    /// ```no_run
    /// use easel::*;
    /// use enigo::*;
    /// use std::time::Duration;
    ///
    /// let mut easel = Easel::new("coords.json", Enigo::new(),
    ///     Duration::from_millis(6));
    /// easel.change_brush_size(0); // shrinks the brush to it's minimum size
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
        self.move_and_click(brush_coords.0, brush_coords.1);
        for _ in 1..num_clicks {
            self.click(Some(mouse_wait));
            thread::sleep(mouse_wait);
        }
        self.brush_size = brush_size;
    }

    /// Draws a single pixel as the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `coords`: The coordinates of the pixel from the input image.
    /// * `color`: The RBGA color to draw to the pixel.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use easel::*;
    /// use enigo::*;
    /// use std::time::Duration;
    /// use image::Rgba;
    ///
    /// // Draw a black pixel in the upper-left corner of the easel.
    /// let mut easel = Easel::new("coords.json", Enigo::new(),
    ///     Duration::from_millis(6));
    /// let color = Rgba { data = [0, 0, 0, 255] };
    /// let coords = (0, 0);
    /// easel.draw_pixel(coords, &color);
    /// ```
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
    /// use easel::*;
    /// use enigo::*;
    /// use std::time::Duration;
    /// use image::Rgba;
    ///
    /// // Draw a black diagonal line from the upper-left corner
    /// // 100 pixels away.
    /// let mut easel = Easel::new("coords.json", Enigo::new(),
    ///     Duration::from_millis(6));
    /// let color = Rgba { data = [0, 0, 0, 255] };
    /// let start = (0, 0);
    /// let end = (100, 100);
    /// easel.draw_line(start, end, &color);
    /// ```
    pub fn draw_line(
        &mut self,
        start_line: Point,
        end_line: Point,
        color: &PaletteColor,
    ) -> Result<(), Error> {
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
        points: &[Point],
        color: &PaletteColor,
        close_shape: bool,
        fill: bool,
    ) -> Result<(), Error> {
        let (start, end) = self.get_bounds();
        self.change_color(color);

        let start_point = match points.get(0) {
            Some(p) => p,
            None => Err(EaselError::NoPoints)?,
        };

        let start_point =
            ((start.0 + start_point.0), (start.1 + start_point.1));
        if start_point.0 > end.0 || start_point.1 > end.1 {
            Err(EaselError::OutOfBounds)?
        }

        self.mouse.mouse_move_to(start_point.0, start_point.1);
        thread::sleep(self.mouse_wait);
        self.mouse.mouse_down(MouseButton::Left);
        for point in points.iter() {
            let point = ((start.0 + point.0), (start.1 + point.1));
            if point.0 > end.0 || point.1 > end.1 {
                Err(EaselError::OutOfBounds)?
            }
            self.mouse.mouse_move_to(point.0, point.1);
            thread::sleep(self.mouse_wait);
        }

        if close_shape || fill {
            self.mouse.mouse_move_to(start_point.0, start_point.1);
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
    fn fill(
        &mut self,
        points: &[Point],
        color: &PaletteColor,
    ) -> Result<(), Error> {
        let mut edges: Vec<[Point; 2]> =
            points.windows(2).map(|pts| [pts[0], pts[1]]).collect();
        edges.push([*points.first().unwrap(), *points.last().unwrap()]);
        let slope: Vec<f32> = edges
            .iter()
            .map(|pts| {
                if pts[1].0 == pts[0].0 {
                    0.0
                } else {
                    (pts[1].1 - pts[0].1) as f32 / (pts[1].0 - pts[0].0) as f32
                }
            })
            .collect();
        let start_y = points.iter().fold(10_000, |acc, pnt| acc.min(pnt.1));
        let end_y = points.iter().fold(0, |acc, pnt| acc.max(pnt.1));
        let mut iy = start_y;
        let old_brush_size = self.brush_size;
        self.change_brush_size(0);

        while iy < end_y {
            let mut active_edges = vec![];
            for (edge, slope) in edges.iter().zip(slope.iter()) {
                let max = edge[0].1.max(edge[1].1);
                let min = edge[0].1.min(edge[1].1);
                if max > iy && min < iy && max != min {
                    active_edges.push((edge, slope));
                }
            }
            let x_draw: Vec<i32> = active_edges
                .iter()
                .map(|&(pts, m)| {
                    if *m == 0.0 {
                        pts[0].0
                    } else {
                        (pts[0].0 as f32 + 1.0 / m * (iy - pts[0].1) as f32)
                            as i32
                    }
                })
                .collect();
            let mut in_poly = true;
            for x in x_draw.windows(2) {
                if x[0] != x[1] && in_poly {
                    self.draw_line((x[0], iy), (x[1], iy), color)?;
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
