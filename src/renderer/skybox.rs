use super::framebuffer::{Color, Framebuffer};
use super::shaders::fbm;
use crate::engine::camera::Camera;
use crate::engine::math::Vec3;
use std::f32::consts::TAU;

const MIN_BRIGHTNESS: f32 = 0.2;
const TWINKLE_AMOUNT: f32 = 0.25;
const MIN_TWINKLE_SPEED: f32 = 1.0;
const TWINKLE_SPEED_RANGE: f32 = 3.0;
const NEBULA_CELL: usize = 4;
const NEBULA_PURPLE: Vec3 = Vec3::new(0.30, 0.10, 0.48);
const NEBULA_BLUE: Vec3 = Vec3::new(0.08, 0.16, 0.45);
const NEBULA_PURPLE_SCALE: f32 = 1.8;
const NEBULA_BLUE_SCALE: f32 = 2.6;
const NEBULA_BLUE_OFFSET: Vec3 = Vec3::new(17.3, -4.1, 9.7);
const NEBULA_THRESHOLD: f32 = 0.5;
const NEBULA_SOFTNESS: f32 = 0.38;
const NEBULA_STRENGTH: f32 = 0.32;

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
    nebula_grid: Vec<Vec3>,
}

fn nebula_density(direction: Vec3, scale: f32, offset: Vec3) -> f32 {
    let value = fbm(direction * scale + offset);
    let t = ((value - NEBULA_THRESHOLD) / NEBULA_SOFTNESS).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn nebula_color(direction: Vec3) -> Vec3 {
    let purple = nebula_density(direction, NEBULA_PURPLE_SCALE, Vec3::default());
    let blue = nebula_density(direction, NEBULA_BLUE_SCALE, NEBULA_BLUE_OFFSET);
    (NEBULA_PURPLE * purple + NEBULA_BLUE * blue) * NEBULA_STRENGTH
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
            nebula_grid: Vec::new(),
        }
    }

    pub fn visible_stars(&self) -> &[VisibleStar] {
        &self.visible
    }

    pub fn draw_nebula(&mut self, framebuffer: &mut Framebuffer, camera: &Camera) {
        let (forward, right, up) = (camera.forward(), camera.right(), camera.up());
        let half_height = (camera.fov_y * 0.5).tan();
        let half_width = half_height * camera.aspect;
        let (width, height) = (framebuffer.width(), framebuffer.height());
        let columns = width / NEBULA_CELL + 2;
        let rows = height / NEBULA_CELL + 2;

        self.nebula_grid.clear();
        for row in 0..rows {
            for column in 0..columns {
                let ndc_x = (column * NEBULA_CELL) as f32 / width as f32 * 2.0 - 1.0;
                let ndc_y = 1.0 - (row * NEBULA_CELL) as f32 / height as f32 * 2.0;
                let direction =
                    (forward + right * (ndc_x * half_width) + up * (ndc_y * half_height)).normalize();
                self.nebula_grid.push(nebula_color(direction));
            }
        }

        let cell = NEBULA_CELL as f32;
        for y in 0..height {
            let row = y / NEBULA_CELL;
            let ty = (y % NEBULA_CELL) as f32 / cell;
            for x in 0..width {
                let column = x / NEBULA_CELL;
                let tx = (x % NEBULA_CELL) as f32 / cell;
                let at = |r: usize, c: usize| self.nebula_grid[r * columns + c];
                let top = at(row, column) + (at(row, column + 1) - at(row, column)) * tx;
                let bottom =
                    at(row + 1, column) + (at(row + 1, column + 1) - at(row + 1, column)) * tx;
                let color = top + (bottom - top) * ty;
                framebuffer.set_pixel(x as i32, y as i32, Color::from_unit(color));
            }
        }
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
    fn nebula_mixes_purple_and_blue_and_leaves_dark_gaps() {
        let samples: Vec<Vec3> = (0..400)
            .map(|index| {
                let angle = index as f32 * 0.37;
                let height = (index as f32 * 0.011).sin();
                nebula_color(Vec3::new(angle.cos(), height, angle.sin()).normalize())
            })
            .collect();
        assert!(samples.iter().any(|color| color.x > color.y && color.z > 0.05));
        assert!(samples.iter().any(|color| color.length() < 0.01));
        assert!(samples.iter().all(|color| color.x <= 1.0 && color.z <= 1.0));
    }

    #[test]
    fn same_seed_gives_same_sky_and_translation_does_not_move_it() {
        let origin = lit_pixels(Vec3::default());
        assert!(origin.chunks(4).any(|pixel| pixel[0] > 0));
        assert_eq!(origin, lit_pixels(Vec3::default()));
        assert_eq!(origin, lit_pixels(Vec3::new(100.0, -40.0, 8.0)));
    }
}
