extern crate enigo;
extern crate failure;
extern crate image;

use easel::Easel;
use self::image::DynamicImage;
use self::image::FilterType;
use self::image::GenericImage;

pub fn size_to_easel(image: &DynamicImage, easel: &Easel) -> DynamicImage {
    let (size_x, size_y) = image.dimensions();
    let (ul_corner, br_corner) = if size_x > size_y {
        easel.easel_coords.landscape_bounds
    } else {
        easel.easel_coords.portrait_bounds
    };
    let x_bounds = br_corner.0 - ul_corner.0;
    let y_bounds = br_corner.1 - ul_corner.1;
    image.resize(x_bounds as u32, y_bounds as u32, FilterType::Lanczos3)
}
