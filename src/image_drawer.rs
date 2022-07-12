use crate::colors::{Palette, PaletteColor};
use crate::coords::Coord;
use crate::easel::{Easel, Orientation, Tool};
use image::imageops::ColorMap;
use image::imageops::FilterType;
use image::DynamicImage;
use image::GenericImageView;
use image::Rgba;
use std::error::Error;

/// A structure that provides means to draw whole images to the easel.
pub struct ImageDrawer<'a> {
    easel: &'a mut Easel,
    palette: Palette,
    current_color: PaletteColor,

    // The size of the easel along x.
    easel_size: Coord,

    // The starting point to use for the next draw operation.
    current_pos: Coord,

    // The size of the image.
    image_size: Coord,

    // The offset of the image to center it on the easel.
    offset: Coord,
}

/// A helper function for scaling images to the dimensions of the easel prior to drawing.
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
    pub fn new(easel: &'a mut Easel, size_x: u32, size_y: u32) -> ImageDrawer<'a> {
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
        let easel_size = Coord::new(easel_x, easel_y);

        let size_x = size_x as i32;
        let size_y = size_y as i32;
        let image_size = Coord::new(size_x, size_y);
        //
        // Offsets used to center the image as best as possible on the easel.
        let offset_x = (easel_x - size_x + 1) / 2;
        let offset_y = (easel_y - size_y) / 2;
        let offset = Coord::new(offset_x, offset_y);

        let start_x = offset_x;
        let start_y = offset_y;
        let current_pos = Coord::new(start_x, start_y);

        ImageDrawer {
            easel,
            palette: Palette::new(),
            current_color,
            easel_size,
            image_size,
            current_pos,
            offset,
        }
    }

    /// Draw the top white border for centering the image along the y-axis.
    ///
    /// If the image completely fills the y axis of the easel, this method
    /// does no drawing.
    pub fn draw_top_border(&mut self) -> Result<(), Box<dyn Error>> {
        for iy in 0..self.offset.y {
            self.easel.draw_line(
                Coord::new(0, iy),
                Coord::new(self.easel_size.x, iy),
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
    ) -> Result<(), Box<dyn Error>> {
        let mut in_coord = Coord::new(x as i32, y as i32);
        in_coord = in_coord + &self.offset;
        let closest_color = self.palette.colormap[self.palette.index_of(rgba)];

        // If we've hit the end of a row, draw the rest of the row before
        // moving on to the next row.
        if in_coord.y > self.current_pos.y {
            self.easel.draw_line(
                self.current_pos,
                Coord::new(self.image_size.x + self.offset.x, self.current_pos.y),
                &self.current_color,
            )?;
            self.current_pos = in_coord;
            self.current_color = closest_color;
        }

        // If there's a color change, draw the line up to this pixel and stop.
        if closest_color != self.current_color {
            self.easel.draw_line(
                self.current_pos,
                in_coord - &Coord::new(1, 0),
                &self.current_color,
            )?;
            self.current_pos = in_coord;
            self.current_color = closest_color;
        }

        Ok(())
    }

    /// Draw the bottom white border and clean up the horizontal edges.
    pub fn cleanup_image(&mut self) -> Result<(), Box<dyn Error>> {
        // Clean up the left-most edge of the picture if one exists.
        let left_edge = self.offset.x - 1;
        if left_edge > 0 {
            self.easel.draw_line(
                Coord::new(left_edge, 0),
                Coord::new(left_edge, self.image_size.y),
                &PaletteColor::White,
            )?;
            for ix in self.offset.y..self.image_size.y + self.offset.y {
                self.easel.draw_line(
                    Coord::new(0, ix),
                    Coord::new(left_edge, ix),
                    &PaletteColor::White,
                )?;
            }
        }

        // Clean up the right-most edge of the picture if one exists.
        let right_edge = self.image_size.x + self.offset.x + 1;
        if right_edge < self.easel_size.x {
            self.easel.draw_line(
                Coord::new(right_edge, 0),
                Coord::new(right_edge, self.image_size.y),
                &PaletteColor::White,
            )?;
            for ix in self.offset.y..self.image_size.y + self.offset.y {
                self.easel.draw_line(
                    Coord::new(right_edge, ix),
                    Coord::new(self.easel_size.x, ix),
                    &PaletteColor::White,
                )?;
            }
        }

        // Once we've hit the end of the picture, tidy up the bottom by drawing
        // white lines to fill in the entire canvas.
        if self.current_pos.y < self.easel_size.y {
            for iy in self.current_pos.y..self.easel_size.y {
                self.easel.draw_line(
                    Coord::new(0, iy),
                    Coord::new(self.easel_size.x, iy),
                    &PaletteColor::White,
                )?;
            }
        }

        Ok(())
    }
}
