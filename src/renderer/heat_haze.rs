use super::dithering::threshold;
use super::framebuffer::{Color, Framebuffer};
use super::outline::{ObjectBuffer, NO_OBJECT};
use super::shaders::ShaderKind;
use super::RenderObject;

const BYTES_PER_PIXEL: usize = 4;
const DISTORTION_OUTER_RADIUS: f32 = 1.42;
const GLOW_OUTER_RADIUS: f32 = 2.25;
const MAX_SHIFT: f32 = 3.2;
const MIN_HALF_EXTENT: f32 = 2.0;
const MAX_GLOW: f32 = 0.62;

#[derive(Debug, Clone, Copy)]
struct Bounds {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

impl Bounds {
    fn include(&mut self, x: i32, y: i32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    fn center(self) -> (f32, f32) {
        (
            (self.min_x + self.max_x) as f32 * 0.5,
            (self.min_y + self.max_y) as f32 * 0.5,
        )
    }

    fn half_extents(self) -> (f32, f32) {
        (
            ((self.max_x - self.min_x) as f32 * 0.5).max(MIN_HALF_EXTENT),
            ((self.max_y - self.min_y) as f32 * 0.5).max(MIN_HALF_EXTENT),
        )
    }
}

pub struct HeatHaze {
    source: Vec<u8>,
}

impl HeatHaze {
    pub fn new() -> Self {
        Self { source: Vec::new() }
    }

    pub fn apply(
        &mut self,
        framebuffer: &mut Framebuffer,
        object_buffer: &ObjectBuffer,
        objects: &[RenderObject],
        time: f32,
    ) {
        let sun_ids: Vec<u32> = objects
            .iter()
            .enumerate()
            .filter(|(_, object)| object.material.shader == ShaderKind::Sun)
            .map(|(index, _)| index as u32 + 1)
            .collect();
        if sun_ids.is_empty() {
            return;
        }

        self.source.clear();
        self.source.extend_from_slice(framebuffer.pixels());
        for sun_id in sun_ids {
            if let Some(bounds) = find_bounds(object_buffer, framebuffer, sun_id) {
                self.distort_sun(framebuffer, object_buffer, sun_id, bounds, time);
            }
        }
    }

    fn distort_sun(
        &self,
        framebuffer: &mut Framebuffer,
        object_buffer: &ObjectBuffer,
        sun_id: u32,
        bounds: Bounds,
        time: f32,
    ) {
        let width = framebuffer.width() as i32;
        let height = framebuffer.height() as i32;
        let (center_x, center_y) = bounds.center();
        let (half_width, half_height) = bounds.half_extents();
        let reach_x = (half_width * GLOW_OUTER_RADIUS).ceil() as i32;
        let reach_y = (half_height * GLOW_OUTER_RADIUS).ceil() as i32;
        let min_x = (center_x as i32 - reach_x).max(0);
        let max_x = (center_x as i32 + reach_x).min(width - 1);
        let min_y = (center_y as i32 - reach_y).max(0);
        let max_y = (center_y as i32 + reach_y).min(height - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if object_buffer.get(x, y) == sun_id {
                    continue;
                }
                let dx = (x as f32 - center_x) / half_width;
                let dy = (y as f32 - center_y) / half_height;
                let radius = (dx * dx + dy * dy).sqrt();
                if !(1.0..GLOW_OUTER_RADIUS).contains(&radius) {
                    continue;
                }
                let mut source_x = x;
                let mut source_y = y;
                if radius < DISTORTION_OUTER_RADIUS {
                    let band = (radius - 1.0) / (DISTORTION_OUTER_RADIUS - 1.0);
                    let distortion = (band * std::f32::consts::PI).sin();
                    let shift_x = ((y as f32 * 0.31 + time * 8.0).sin()
                        + (y as f32 * 0.11 - time * 5.3).sin() * 0.5)
                        * MAX_SHIFT
                        * distortion;
                    let shift_y =
                        (x as f32 * 0.23 - time * 6.1).sin() * MAX_SHIFT * 0.45 * distortion;
                    source_x = (x as f32 + shift_x).round().clamp(0.0, (width - 1) as f32) as i32;
                    source_y = (y as f32 + shift_y).round().clamp(0.0, (height - 1) as f32) as i32;
                    if object_buffer.get(source_x, source_y) == sun_id {
                        source_x = x;
                        source_y = y;
                    }
                }
                let glow_t = ((radius - 1.0) / (GLOW_OUTER_RADIUS - 1.0)).clamp(0.0, 1.0);
                let glow = (1.0 - glow_t).powi(2) * MAX_GLOW;
                let color = self.source_color(width, source_x, source_y);
                framebuffer.set_pixel(x, y, add_solar_glow(color, glow, x, y));
            }
        }
    }

    fn source_color(&self, width: i32, x: i32, y: i32) -> Color {
        let start = (y * width + x) as usize * BYTES_PER_PIXEL;
        Color::new(
            self.source[start],
            self.source[start + 1],
            self.source[start + 2],
        )
    }
}

fn add_solar_glow(color: Color, amount: f32, x: i32, y: i32) -> Color {
    if amount <= threshold(x, y) {
        return color;
    }
    let add = |channel: u8, light: f32| {
        (channel as f32 + (255.0 - channel as f32) * light * amount).clamp(0.0, 255.0) as u8
    };
    Color::new(add(color.r, 1.0), add(color.g, 0.62), add(color.b, 0.12))
}

fn find_bounds(
    object_buffer: &ObjectBuffer,
    framebuffer: &Framebuffer,
    object_id: u32,
) -> Option<Bounds> {
    let mut bounds = Bounds {
        min_x: framebuffer.width() as i32,
        min_y: framebuffer.height() as i32,
        max_x: -1,
        max_y: -1,
    };
    for y in 0..framebuffer.height() as i32 {
        for x in 0..framebuffer.width() as i32 {
            if object_buffer.get(x, y) == object_id {
                bounds.include(x, y);
            }
        }
    }
    (bounds.max_x >= 0 && object_id != NO_OBJECT).then_some(bounds)
}

impl Default for HeatHaze {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_haze_preserves_the_emitter_and_pixels_outside_its_aura() {
        let mut framebuffer = Framebuffer::new();
        let mut objects = ObjectBuffer::new();
        for y in 120..150 {
            for x in 220..260 {
                objects.set(x, y, 1);
                framebuffer.set_pixel(x, y, Color::new(255, 90, 0));
            }
        }
        for y in 0..framebuffer.height() as i32 {
            for x in 0..framebuffer.width() as i32 {
                if objects.get(x, y) == NO_OBJECT {
                    framebuffer.set_pixel(x, y, Color::new((x % 255) as u8, (y % 255) as u8, 40));
                }
            }
        }
        let original = framebuffer.pixels().to_vec();
        let bounds = find_bounds(&objects, &framebuffer, 1).unwrap();
        let mut haze = HeatHaze::new();
        haze.source = original.clone();
        haze.distort_sun(&mut framebuffer, &objects, 1, bounds, 0.7);

        let pixel = |pixels: &[u8], x: usize, y: usize| {
            let start = (y * framebuffer.width() + x) * BYTES_PER_PIXEL;
            [pixels[start], pixels[start + 1], pixels[start + 2]]
        };
        assert_eq!(
            pixel(framebuffer.pixels(), 240, 135),
            pixel(&original, 240, 135)
        );
        assert_eq!(
            pixel(framebuffer.pixels(), 20, 20),
            pixel(&original, 20, 20)
        );
        assert!((195..215).any(|x| pixel(framebuffer.pixels(), x, 135) != pixel(&original, x, 135)));
    }

    #[test]
    fn solar_glow_uses_the_same_ordered_dithering_pattern() {
        let background = Color::BLACK;
        let lit = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .filter(|&(x, y)| add_solar_glow(background, 0.5, x, y) != background)
            .count();
        assert_eq!(lit, 8);
    }
}
