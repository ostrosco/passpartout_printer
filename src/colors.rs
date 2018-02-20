extern crate image;

use image::Rgba;
use image::imageops::colorops::ColorMap;
use std::f32;

pub type Point = (i32, i32);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PaletteColor {
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

#[derive(Default)]
pub struct Palette {
    pub colormap: Vec<PaletteColor>,
}

impl PaletteColor {
    pub fn get_row_col(&self) -> Point {
        match *self {
            PaletteColor::Black => (0, 0),
            PaletteColor::Grey => (0, 1),
            PaletteColor::White => (0, 2),
            PaletteColor::DarkBrown => (1, 0),
            PaletteColor::Brown => (1, 1),
            PaletteColor::LightBrown => (1, 2),
            PaletteColor::DarkRed => (2, 0),
            PaletteColor::Red => (2, 1),
            PaletteColor::Pink => (2, 2),
            PaletteColor::Orange => (3, 0),
            PaletteColor::DarkYellow => (3, 1),
            PaletteColor::Yellow => (3, 2),
            PaletteColor::DarkGreen => (4, 0),
            PaletteColor::Green => (4, 1),
            PaletteColor::LightGreen => (4, 2),
            PaletteColor::DarkBlue => (5, 0),
            PaletteColor::Blue => (5, 1),
            PaletteColor::LightBlue => (5, 2),
            PaletteColor::DarkIndigo => (6, 0),
            PaletteColor::Indigo => (6, 1),
            PaletteColor::LightIndigo => (6, 2),
            PaletteColor::DarkViolet => (7, 0),
            PaletteColor::Violet => (7, 1),
            PaletteColor::LightViolet => (7, 2),
        }
    }

    pub fn get_rgba(&self) -> Rgba<u8> {
        let data = match *self {
            PaletteColor::Black => [0x0d, 0x0d, 0x0d, 0x00],
            PaletteColor::Grey => [0x76, 0x76, 0x76, 0x00],
            PaletteColor::White => [0xe5, 0xe5, 0xe5, 0x00],
            PaletteColor::DarkBrown => [0x62, 0x32, 0x00, 0x00],
            PaletteColor::Brown => [0xb9, 0x7a, 0x56, 0x00],
            PaletteColor::LightBrown => [0xef, 0xe4, 0xb0, 0x00],
            PaletteColor::DarkRed => [0x7e, 0x0d, 0x0d, 0x00],
            PaletteColor::Red => [0xed, 0x1c, 0x22, 0x00],
            PaletteColor::Pink => [0xff, 0xae, 0xc9, 0x00],
            PaletteColor::Orange => [0xff, 0x7f, 0x26, 0x00],
            PaletteColor::DarkYellow => [0xff, 0xc9, 0x0d, 0x00],
            PaletteColor::Yellow => [0xfa, 0xed, 0x16, 0x00],
            PaletteColor::DarkGreen => [0x26, 0x5d, 0x38, 0x00],
            PaletteColor::Green => [0x35, 0xab, 0x55, 0x00],
            PaletteColor::LightGreen => [0xb5, 0xe6, 0x1c, 0x00],
            PaletteColor::DarkBlue => [0x00, 0x65, 0x91, 0x00],
            PaletteColor::Blue => [0x00, 0xa2, 0xe8, 0x00],
            PaletteColor::LightBlue => [0x99, 0xd9, 0xea, 0x00],
            PaletteColor::DarkIndigo => [0x1c, 0x22, 0x63, 0x00],
            PaletteColor::Indigo => [0x30, 0x39, 0xcc, 0x00],
            PaletteColor::LightIndigo => [0x70, 0x92, 0xbe, 0x00],
            PaletteColor::DarkViolet => [0x95, 0x35, 0x96, 0x00],
            PaletteColor::Violet => [0xd5, 0x5f, 0xd7, 0x00],
            PaletteColor::LightViolet => [0xc1, 0xa7, 0xd7, 0x00],
        };
        Rgba { data: data }
    }
}

impl Palette {
    pub fn new() -> Palette {
        let colors = vec![
            PaletteColor::Black,
            PaletteColor::Grey,
            PaletteColor::White,
            PaletteColor::DarkBrown,
            PaletteColor::Brown,
            PaletteColor::LightBrown,
            PaletteColor::DarkRed,
            PaletteColor::Red,
            PaletteColor::Pink,
            PaletteColor::Orange,
            PaletteColor::DarkYellow,
            PaletteColor::Yellow,
            PaletteColor::DarkGreen,
            PaletteColor::Green,
            PaletteColor::LightGreen,
            PaletteColor::DarkBlue,
            PaletteColor::Blue,
            PaletteColor::LightBlue,
            PaletteColor::DarkIndigo,
            PaletteColor::Indigo,
            PaletteColor::LightIndigo,
            PaletteColor::DarkViolet,
            PaletteColor::Violet,
            PaletteColor::LightViolet,
        ];
        Palette { colormap: colors }
    }

    fn get_closest_color(&self, color: &Rgba<u8>) -> (usize, Rgba<u8>) {
        let r = f32::from(color[0]);
        let g = f32::from(color[1]);
        let b = f32::from(color[2]);
        let a = f32::from(color[3]);

        let mut index = 0;
        let mut closest_color = PaletteColor::Black;
        let mut color_dist = f32::MAX;

        // Iterate over all colors and compare the RBG values to find the
        // closest value to the input color.
        for (ix, col) in self.colormap.iter().enumerate() {
            let hex = col.get_rgba();
            let col_r = f32::from(hex[0]);
            let col_g = f32::from(hex[1]);
            let col_b = f32::from(hex[2]);
            let col_a = f32::from(hex[3]);
            let col_r_diff =
                (col_r - r).powi(2).max((col_r - r - col_a + a).powi(2));
            let col_g_diff =
                (col_g - g).powi(2).max((col_g - g - col_a + a).powi(2));
            let col_b_diff =
                (col_b - b).powi(2).max((col_b - b - col_a + a).powi(2));
            let curr_color_dist = (col_r_diff + col_g_diff + col_b_diff).sqrt();
            if curr_color_dist < color_dist {
                index = ix;
                closest_color = *col;
                color_dist = curr_color_dist;
            }
        }
        (index, closest_color.get_rgba())
    }
}

impl ColorMap for Palette {
    type Color = Rgba<u8>;

    fn map_color(&self, color: &mut Self::Color) {
        let (_, closest_rgba) = self.get_closest_color(color);
        color.data[0] = closest_rgba[0];
        color.data[1] = closest_rgba[1];
        color.data[2] = closest_rgba[2];
        color.data[3] = closest_rgba[3];
    }

    fn index_of(&self, color: &Self::Color) -> usize {
        let (index, _) = self.get_closest_color(color);
        index
    }
}
