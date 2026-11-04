use crate::engine::math::Vec3;

pub const FRAMEBUFFER_WIDTH: usize = 480;
pub const FRAMEBUFFER_HEIGHT: usize = 270;

const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Self = Self::new(0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_unit(color: Vec3) -> Self {
        let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0) as u8;
        Self::new(channel(color.x), channel(color.y), channel(color.z))
    }
}

pub struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn new() -> Self {
        let mut framebuffer = Self {
            width: FRAMEBUFFER_WIDTH,
            height: FRAMEBUFFER_HEIGHT,
            pixels: vec![0; FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT * BYTES_PER_PIXEL],
        };
        framebuffer.clear(Color::BLACK);
        framebuffer
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn clear(&mut self, color: Color) {
        for pixel in self.pixels.chunks_exact_mut(BYTES_PER_PIXEL) {
            pixel.copy_from_slice(&[color.r, color.g, color.b, 255]);
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let start = (y as usize * self.width + x as usize) * BYTES_PER_PIXEL;
        self.pixels[start..start + BYTES_PER_PIXEL]
            .copy_from_slice(&[color.r, color.g, color.b, 255]);
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Color::BLACK;
        }
        let start = (y as usize * self.width + x as usize) * BYTES_PER_PIXEL;
        Color::new(
            self.pixels[start],
            self.pixels[start + 1],
            self.pixels[start + 2],
        )
    }
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}
