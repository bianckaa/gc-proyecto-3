use super::framebuffer::{Color, Framebuffer};
use super::lines::draw_line_2d;
use super::skybox::VisibleStar;
use std::f32::consts::PI;

const WARP_DURATION: f32 = 2.5;
const DISTORTION_STRENGTH: f32 = 0.35;
const STREAK_LENGTH_FACTOR: f32 = 0.7;
const MIN_STREAK_LENGTH: f32 = 8.0;
const FLASH_START: f32 = 0.75;
const MAX_FLASH: f32 = 0.85;
const BYTES_PER_PIXEL: usize = 4;

fn flash_amount(intensity: f32) -> f32 {
    ((intensity - FLASH_START) / (1.0 - FLASH_START)).max(0.0) * MAX_FLASH
}

fn brighten(channel: u8, flash: f32) -> u8 {
    (channel as f32 + (255.0 - channel as f32) * flash) as u8
}

pub struct WarpEffect {
    elapsed: Option<f32>,
    source: Vec<u8>,
}

impl WarpEffect {
    pub fn new() -> Self {
        Self {
            elapsed: None,
            source: Vec::new(),
        }
    }

    pub fn start_warp(&mut self) {
        self.elapsed = Some(0.0);
    }

    pub fn update(&mut self, delta_time: f32) {
        self.elapsed = self
            .elapsed
            .map(|elapsed| elapsed + delta_time)
            .filter(|&elapsed| elapsed < WARP_DURATION);
    }

    pub fn is_active(&self) -> bool {
        self.elapsed.is_some()
    }

    pub fn intensity(&self) -> f32 {
        self.elapsed
            .map_or(0.0, |elapsed| (elapsed / WARP_DURATION * PI).sin())
    }

    fn distort(&mut self, framebuffer: &mut Framebuffer, intensity: f32, flash: f32) {
        let width = framebuffer.width() as i32;
        let height = framebuffer.height() as i32;
        let (center_x, center_y) = (width as f32 * 0.5, height as f32 * 0.5);
        let max_radius_squared = center_x * center_x + center_y * center_y;

        self.source.clear();
        self.source.extend_from_slice(framebuffer.pixels());

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let radius_squared = (dx * dx + dy * dy) / max_radius_squared;
                let pull = 1.0 - DISTORTION_STRENGTH * intensity * radius_squared;
                let source_x = (center_x + dx * pull) as i32;
                let source_y = (center_y + dy * pull) as i32;
                let start = (source_y * width + source_x) as usize * BYTES_PER_PIXEL;
                let color = Color::new(
                    brighten(self.source[start], flash),
                    brighten(self.source[start + 1], flash),
                    brighten(self.source[start + 2], flash),
                );
                framebuffer.set_pixel(x, y, color);
            }
        }
    }

    pub fn apply(&mut self, framebuffer: &mut Framebuffer, stars: &[VisibleStar]) {
        if !self.is_active() {
            return;
        }
        let intensity = self.intensity();
        let flash = flash_amount(intensity);
        self.distort(framebuffer, intensity, flash);

        let center_x = framebuffer.width() as f32 * 0.5;
        let center_y = framebuffer.height() as f32 * 0.5;
        for star in stars {
            let dx = star.x as f32 - center_x;
            let dy = star.y as f32 - center_y;
            let length = (MIN_STREAK_LENGTH + (dx * dx + dy * dy).sqrt() * STREAK_LENGTH_FACTOR)
                * intensity;
            let radius = (dx * dx + dy * dy).sqrt().max(1.0);
            let end_x = star.x + (dx / radius * length) as i32;
            let end_y = star.y + (dy / radius * length) as i32;
            let color = Color::new(
                brighten(star.color.r, flash),
                brighten(star.color.g, flash),
                brighten(star.color.b, flash),
            );
            draw_line_2d(framebuffer, star.x, star.y, end_x, end_y, color);
        }
    }
}

impl Default for WarpEffect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warp_rises_falls_and_ends_with_time() {
        let mut warp = WarpEffect::new();
        assert!(!warp.is_active());
        warp.start_warp();
        warp.update(WARP_DURATION * 0.5);
        assert!((warp.intensity() - 1.0).abs() < 0.001);
        warp.update(WARP_DURATION);
        assert!(!warp.is_active());
        assert_eq!(warp.intensity(), 0.0);
    }
}
