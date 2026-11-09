use super::framebuffer::{Color, Framebuffer};
use crate::engine::math::{Mat4, Vec3};

const BYTES_PER_PIXEL: usize = 4;
const MAX_HORIZON_PIXELS: f32 = 46.0;
const SWIRL: f32 = 1.4;
const LENS: f32 = 0.9;
const RING_INNER: f32 = 1.05;
const RING_OUTER: f32 = 1.7;
const RING_ARMS: f32 = 6.0;
const RING_SPEED: f32 = 3.0;
const MAX_DARKENING: f32 = 0.55;
const RING_COLOR: Vec3 = Vec3::new(1.0, 0.55, 0.15);
const RING_HOT: Vec3 = Vec3::new(1.0, 0.92, 0.7);

pub struct BlackHoleEffect {
    pub intensity: f32,
    pub center: Vec3,
    source: Vec<u8>,
}

impl BlackHoleEffect {
    pub fn new() -> Self {
        Self {
            intensity: 0.0,
            center: Vec3::default(),
            source: Vec::new(),
        }
    }

    pub fn horizon_radius(&self) -> f32 {
        MAX_HORIZON_PIXELS * self.intensity.clamp(0.0, 1.0)
    }

    pub fn apply(&mut self, framebuffer: &mut Framebuffer, view_projection: Mat4, time: f32) {
        let intensity = self.intensity.clamp(0.0, 1.0);
        if intensity <= 0.0 {
            return;
        }
        let width = framebuffer.width() as i32;
        let height = framebuffer.height() as i32;
        let clip = view_projection * self.center.extend(1.0);
        let visible = clip.w > 0.0;
        let center_x = if visible {
            (clip.x / clip.w + 1.0) * 0.5 * width as f32
        } else {
            width as f32 * 0.5
        };
        let center_y = if visible {
            (1.0 - clip.y / clip.w) * 0.5 * height as f32
        } else {
            height as f32 * 0.5
        };
        let horizon = if visible { self.horizon_radius() } else { 0.0 };

        self.source.clear();
        self.source.extend_from_slice(framebuffer.pixels());
        let sample = |x: i32, y: i32| {
            let x = x.clamp(0, width - 1);
            let y = y.clamp(0, height - 1);
            let start = (y * width + x) as usize * BYTES_PER_PIXEL;
            Vec3::new(
                self.source[start] as f32 / 255.0,
                self.source[start + 1] as f32 / 255.0,
                self.source[start + 2] as f32 / 255.0,
            )
        };
        let darkening = 1.0 - MAX_DARKENING * intensity;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 + 0.5 - center_x;
                let dy = y as f32 + 0.5 - center_y;
                let radius = (dx * dx + dy * dy).sqrt().max(0.001);
                if radius < horizon {
                    framebuffer.set_pixel(x, y, Color::BLACK);
                    continue;
                }
                let pull = if horizon > 0.0 { horizon / radius } else { 0.0 };
                let angle = SWIRL * pull * intensity;
                let (sin, cos) = angle.sin_cos();
                let scale = 1.0 + LENS * pull;
                let source_x = center_x + (dx * cos - dy * sin) * scale;
                let source_y = center_y + (dx * sin + dy * cos) * scale;
                let mut color = sample(source_x as i32, source_y as i32) * darkening;
                let ring = radius / horizon.max(0.001);
                if horizon > 0.0 && (RING_INNER..RING_OUTER).contains(&ring) {
                    let middle = (RING_INNER + RING_OUTER) * 0.5;
                    let falloff = 1.0 - (ring - middle).abs() / ((RING_OUTER - RING_INNER) * 0.5);
                    let swirl = dy.atan2(dx) * RING_ARMS + time * RING_SPEED;
                    let glow = falloff * (0.55 + 0.45 * swirl.sin()) * intensity;
                    let tint = RING_COLOR + (RING_HOT - RING_COLOR) * falloff;
                    color = color + tint * glow;
                }
                framebuffer.set_pixel(x, y, Color::from_unit(color));
            }
        }
    }
}

impl Default for BlackHoleEffect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white_frame() -> Framebuffer {
        let mut framebuffer = Framebuffer::new();
        framebuffer.clear(Color::new(255, 255, 255));
        framebuffer
    }

    #[test]
    fn horizon_swallows_the_center_and_darkens_the_rest() {
        let mut framebuffer = white_frame();
        let mut effect = BlackHoleEffect::new();
        effect.intensity = 1.0;
        effect.center = Vec3::new(0.0, 0.0, -5.0);
        let projection = Mat4::perspective(1.0, 16.0 / 9.0, 0.1, 100.0);
        effect.apply(&mut framebuffer, projection, 0.0);
        let center = framebuffer.get_pixel(240, 135);
        let corner = framebuffer.get_pixel(2, 2);
        assert_eq!(center, Color::BLACK);
        assert!(corner.r < 255 && corner.r > 0);
    }

    #[test]
    fn zero_intensity_leaves_the_frame_untouched() {
        let mut framebuffer = white_frame();
        let mut effect = BlackHoleEffect::new();
        effect.apply(&mut framebuffer, Mat4::identity(), 0.0);
        assert_eq!(framebuffer.get_pixel(240, 135), Color::new(255, 255, 255));
    }
}
