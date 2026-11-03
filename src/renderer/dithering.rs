use super::framebuffer::Color;
use super::shaders::Surface;

const BAYER_SIZE: usize = 4;
const BAYER_LEVELS: f32 = 16.0;
const BAYER_MATRIX: [[u8; BAYER_SIZE]; BAYER_SIZE] = [
    [0, 8, 2, 10],
    [12, 4, 14, 6],
    [3, 11, 1, 9],
    [15, 7, 13, 5],
];

fn threshold(x: i32, y: i32) -> f32 {
    let row = y.rem_euclid(BAYER_SIZE as i32) as usize;
    let column = x.rem_euclid(BAYER_SIZE as i32) as usize;
    (BAYER_MATRIX[row][column] as f32 + 0.5) / BAYER_LEVELS
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
    use crate::engine::math::Vec3;

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
    fn dot_density_follows_intensity() {
        assert_eq!(lit_pixels(0.0), 0);
        assert_eq!(lit_pixels(0.5), 8);
        assert_eq!(lit_pixels(1.0), 16);
    }
}
