use super::framebuffer::{Color, Framebuffer};
use crate::engine::camera::Camera;
use crate::engine::math::Vec3;
use std::f32::consts::TAU;

const MIN_BRIGHTNESS: f32 = 0.2;
const TWINKLE_AMOUNT: f32 = 0.25;
const MIN_TWINKLE_SPEED: f32 = 1.0;
const TWINKLE_SPEED_RANGE: f32 = 3.0;

struct Star {
    direction: Vec3,
    brightness: f32,
    twinkle_phase: f32,
    twinkle_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibleStar {
    pub x: i32,
    pub y: i32,
    pub color: Color,
}

pub struct Skybox {
    stars: Vec<Star>,
    visible: Vec<VisibleStar>,
}

pub(crate) struct SeededRandom {
    pub(crate) state: u32,
}

impl SeededRandom {
    pub(crate) fn next(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.state >> 8) as f32 / (1u32 << 24) as f32
    }
}

impl Skybox {
    pub fn new(seed: u32, star_count: usize) -> Self {
        let mut random = SeededRandom { state: seed };
        let stars = (0..star_count)
            .map(|_| {
                let height = random.next() * 2.0 - 1.0;
                let azimuth = random.next() * TAU;
                let ring = (1.0 - height * height).sqrt();
                let brightness = random.next();
                Star {
                    direction: Vec3::new(ring * azimuth.cos(), height, ring * azimuth.sin()),
                    brightness: MIN_BRIGHTNESS + (1.0 - MIN_BRIGHTNESS) * brightness * brightness,
                    twinkle_phase: random.next() * TAU,
                    twinkle_speed: MIN_TWINKLE_SPEED + random.next() * TWINKLE_SPEED_RANGE,
                }
            })
            .collect();
        Self {
            stars,
            visible: Vec::new(),
        }
    }

    pub fn visible_stars(&self) -> &[VisibleStar] {
        &self.visible
    }

    pub fn draw(&mut self, framebuffer: &mut Framebuffer, camera: &Camera, time: f32) {
        let (forward, right, up) = (camera.forward(), camera.right(), camera.up());
        let focal = 1.0 / (camera.fov_y * 0.5).tan();
        let half_width = framebuffer.width() as f32 * 0.5;
        let half_height = framebuffer.height() as f32 * 0.5;

        self.visible.clear();
        for star in &self.stars {
            let depth = star.direction.dot(forward);
            if depth <= 0.0 {
                continue;
            }
            let ndc_x = star.direction.dot(right) / depth * focal / camera.aspect;
            let ndc_y = star.direction.dot(up) / depth * focal;
            let x = ((ndc_x + 1.0) * half_width) as i32;
            let y = ((1.0 - ndc_y) * half_height) as i32;
            let twinkle = (time * star.twinkle_speed + star.twinkle_phase).sin();
            let level = star.brightness * (1.0 - TWINKLE_AMOUNT * (twinkle * 0.5 + 0.5));
            let color = Color::from_unit(Vec3::new(level, level, level));
            framebuffer.set_pixel(x, y, color);
            self.visible.push(VisibleStar { x, y, color });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit_pixels(position: Vec3) -> Vec<u8> {
        let mut framebuffer = Framebuffer::new();
        let mut skybox = Skybox::new(7, 500);
        skybox.draw(&mut framebuffer, &Camera::new(position, 16.0 / 9.0), 0.0);
        framebuffer.pixels().to_vec()
    }

    #[test]
    fn same_seed_gives_same_sky_and_translation_does_not_move_it() {
        let origin = lit_pixels(Vec3::default());
        assert!(origin.chunks(4).any(|pixel| pixel[0] > 0));
        assert_eq!(origin, lit_pixels(Vec3::default()));
        assert_eq!(origin, lit_pixels(Vec3::new(100.0, -40.0, 8.0)));
    }
}
