extern crate enigo;
extern crate failure;
extern crate image;

use easel::{Easel, Orientation, Tool};
use colors::{Palette, PaletteColor};
use coords::Coord;
use self::image::DynamicImage;
use self::image::FilterType;
use self::image::GenericImage;
use self::image::Rgba;
use self::failure::Error;
use image::imageops::ColorMap;

pub struct ImageDrawer<'a> {
    easel: &'a mut Easel,
    palette: Palette,
    current_color: PaletteColor,

    // The size of the easel along x.
    easel_x: i32,
    easel_y: i32,

    // The starting point to use for the next draw operation.
    start_x: i32,
    start_y: i32,

    // The size of the image.
    size_x: i32,
    size_y: i32,

    // The offset of the image to center it on the easel.
    offset_x: i32,
    offset_y: i32,
}

pub fn size_to_easel(image: &DynamicImage, easel: &Easel) -> DynamicImage {
    let (size_x, size_y) = image.dimensions();
    let (ul_corner, br_corner) = if size_x > size_y {
        easel.easel_coords.landscape_bounds
    } else {
        easel.easel_coords.portrait_bounds
    };
    let x_bounds = br_corner.x - ul_corner.x;
    let y_bounds = br_corner.y - ul_corner.y;
    image.resize(x_bounds as u32, y_bounds as u32, FilterType::Lanczos3)
}

impl<'a> ImageDrawer<'a> {
    pub fn new(
        easel: &'a mut Easel,
        size_x: u32,
        size_y: u32,
    ) -> ImageDrawer<'a> {
        // For drawing images, we need the brush to be as small as possible.
        easel.change_brush_size(0);
        easel.change_tool(Tool::Paintbrush);

        if (size_x > size_y && easel.orientation == Orientation::Portrait)
            || (size_y > size_x && easel.orientation == Orientation::Landscape)
        {
            easel.change_orientation();
        }

        let current_color = easel.current_color;

        let (ulcorner, lrcorner) = easel.get_bounds();
        let easel_x = lrcorner.x - ulcorner.x - 1;
        let easel_y = lrcorner.y - ulcorner.y - 1;

        let size_x = size_x as i32;
        let size_y = size_y as i32;
        //
        // Offsets used to center the image as best as possible on the easel.
        let offset_x = (easel_x - size_x + 1) / 2;
        let offset_y = (easel_y - size_y) / 2;

        let start_x = offset_x;
        let start_y = offset_y;

        ImageDrawer {
            easel: easel,
            palette: Palette::new(),
            current_color: current_color,
            easel_x: easel_x,
            easel_y: easel_y,
            start_x: start_x,
            start_y: start_y,
            size_x: size_x,
            size_y: size_y,
            offset_x: offset_x,
            offset_y: offset_y,
        }
    }

    /// Draw the top white border for centering the image along the y-axis.
    ///
    /// If the image completely fills the y axis of the easel, this method
    /// does no drawing.
    pub fn draw_top_border(&mut self) -> Result<(), Error> {
        for iy in 0..self.offset_y {
            self.easel.draw_line(
                Coord::new(0, iy),
                Coord::new(self.easel_x, iy),
                &PaletteColor::White,
            )?;
        }
        self.current_color = self.easel.current_color;
        Ok(())
    }

    /// Process the next pixel from a given image.
    ///
    /// Pixels are not drawn to the screen unless we've hit the end of a row
    /// and must draw or there is a color change where we will draw everything
    /// up to the current pixel. If you want to draw a pixel by itself, use
    /// Easel::draw_pixel() instead.
    ///
    /// # Arguments
    ///
    /// * `rgba`: The RGBA pixel to handle.
    /// * `x`: The x coordinate of the pixel in image coordinates.
    /// * `y`: The x coordinate of the pixel in image coordinates.
    ///
    pub fn handle_pixel(
        &mut self,
        rgba: &mut Rgba<u8>,
        x: u32,
        y: u32,
    ) -> Result<(), Error> {
        let x = x as i32 + self.offset_x;
        let y = y as i32 + self.offset_y;
        let closest_color = self.palette.colormap[self.palette.index_of(rgba)];

        // If we've hit the end of a row, draw the rest of the row before
        // moving on to the next row.
        if y > self.start_y {
            self.easel.draw_line(
                Coord::new(self.start_x, self.start_y),
                Coord::new(self.size_x + self.offset_x, self.start_y),
                &self.current_color,
            )?;
            self.start_x = x;
            self.start_y = y;
            self.current_color = closest_color;
        }

        // If there's a color change, draw the line up to this pixel and stop.
        if closest_color != self.current_color {
            self.easel.draw_line(
                Coord::new(self.start_x, self.start_y),
                Coord::new(x - 1, y),
                &self.current_color,
            )?;
            self.start_x = x;
            self.start_y = y;
            self.current_color = closest_color;
        }

        Ok(())
    }

    /// Draw the bottom white border and clean up the horizontal edges.
    pub fn cleanup_image(&mut self) -> Result<(), Error> {
        // Clean up the left-most edge of the picture if one exists.
        let left_edge = self.offset_x - 1;
        if left_edge > 0 {
            self.easel.draw_line(
                Coord::new(left_edge, 0),
                Coord::new(left_edge, self.size_y),
                &PaletteColor::White,
            )?;
            for ix in self.offset_y..self.size_y + self.offset_y {
                self.easel.draw_line(
                    Coord::new(0, ix),
                    Coord::new(left_edge, ix),
                    &PaletteColor::White,
                )?;
            }
        }

        // Clean up the right-most edge of the picture if one exists.
        let right_edge = self.size_x + self.offset_x + 1;
        if right_edge < self.easel_x {
            self.easel.draw_line(
                Coord::new(right_edge, 0),
                Coord::new(right_edge, self.size_y),
                &PaletteColor::White,
            )?;
            for ix in self.offset_y..self.size_y + self.offset_y {
                self.easel.draw_line(
                    Coord::new(right_edge, ix),
                    Coord::new(self.easel_x, ix),
                    &PaletteColor::White,
                )?;
            }
        }

        // Once we've hit the end of the picture, tidy up the bottom by drawing
        // white lines to fill in the entire canvas.
        if self.start_y < self.easel_y {
            for iy in self.start_y..self.easel_y {
                self.easel.draw_line(
                    Coord::new(0, iy),
                    Coord::new(self.easel_x, iy),
                    &PaletteColor::White,
                )?;
            }
        }

        Ok(())
    }
}
