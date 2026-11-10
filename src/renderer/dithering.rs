use super::framebuffer::Color;
use super::shaders::Surface;
use crate::engine::math::Vec3;

const BAYER_SIZE: usize = 4;
const BAYER_LEVELS: f32 = 16.0;
const BAYER_MATRIX: [[u8; BAYER_SIZE]; BAYER_SIZE] = [
    [0, 8, 2, 10],
    [12, 4, 14, 6],
    [3, 11, 1, 9],
    [15, 7, 13, 5],
];

pub(crate) fn threshold(x: i32, y: i32) -> f32 {
    let row = y.rem_euclid(BAYER_SIZE as i32) as usize;
    let column = x.rem_euclid(BAYER_SIZE as i32) as usize;
    (BAYER_MATRIX[row][column] as f32 + 0.5) / BAYER_LEVELS
}

pub fn screen_door_visible(opacity: f32, x: i32, y: i32) -> bool {
    opacity > threshold(x + 2, y + 1)
}

pub fn dither_levels(color: Vec3, levels: u32, x: i32, y: i32) -> Color {
    let steps = levels.max(2) as f32 - 1.0;
    let limit = threshold(x, y);
    let channel = |value: f32| {
        let scaled = value.clamp(0.0, 1.0) * steps;
        let base = scaled.floor();
        let level = if scaled - base > limit { base + 1.0 } else { base };
        level / steps
    };
    Color::from_unit(Vec3::new(channel(color.x), channel(color.y), channel(color.z)))
}

pub fn dither(surface: Surface, x: i32, y: i32) -> Color {
    if surface.intensity > threshold(x, y) {
        Color::from_unit(surface.color)
    } else {
        Color::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit_pixels(intensity: f32) -> usize {
        let surface = Surface {
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity,
        };
        (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .filter(|&(x, y)| dither(surface, x, y) != Color::BLACK)
            .count()
    }

    #[test]
    fn multi_level_dithering_averages_to_the_original_tone() {
        let tone = Vec3::new(0.43, 0.43, 0.43);
        let average: f32 = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .map(|(x, y)| dither_levels(tone, 6, x, y).r as f32 / 255.0)
            .sum::<f32>()
            / 16.0;
        assert!((average - 0.43).abs() < 0.04);
        assert_eq!(dither_levels(Vec3::new(1.0, 0.0, 0.0), 6, 3, 1), Color::new(255, 0, 0));
    }

    #[test]
    fn dot_density_follows_intensity() {
        assert_eq!(lit_pixels(0.0), 0);
        assert_eq!(lit_pixels(0.5), 8);
        assert_eq!(lit_pixels(1.0), 16);
    }

    #[test]
    fn screen_door_coverage_grows_with_opacity() {
        let coverage = |opacity: f32| {
            (0..4)
                .flat_map(|y| (0..4).map(move |x| (x, y)))
                .filter(|&(x, y)| screen_door_visible(opacity, x, y))
                .count()
        };
        assert_eq!(coverage(0.0), 0);
        assert!(coverage(0.25) < coverage(0.75));
        assert_eq!(coverage(1.0), 16);
    }

    #[test]
    fn screen_door_transparency_preserves_half_the_background() {
        let visible = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .filter(|&(x, y)| screen_door_visible(0.5, x, y))
            .count();
        assert_eq!(visible, 8);
    }
}
