use crate::assets::manager::asset_path;
use crate::renderer::framebuffer::Color;
use crate::renderer::opening_crawl::RasterImage;
use raylib::prelude::Image;
use std::path::Path;

pub const ENDING_IMAGE_FILE: &str = "assets/images/absolute_cinema.png";

pub fn load_raster(relative: &str) -> Option<RasterImage> {
    let path = asset_path(Path::new(relative))?;
    let image = Image::load_image(path.to_str()?).ok()?;
    let width = image.width().max(0) as usize;
    let height = image.height().max(0) as usize;
    let pixels: Vec<Color> = image
        .get_image_data()
        .iter()
        .map(|pixel| Color::new(pixel.r, pixel.g, pixel.b))
        .collect();
    (pixels.len() == width * height && width > 0).then_some(RasterImage {
        width,
        height,
        pixels,
    })
}
