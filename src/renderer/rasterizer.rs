use super::depth::DepthBuffer;
use super::framebuffer::{Color, Framebuffer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenVertex {
    pub x: f32,
    pub y: f32,
    pub depth: f32,
}

impl ScreenVertex {
    pub const fn new(x: f32, y: f32, depth: f32) -> Self {
        Self { x, y, depth }
    }
}

fn edge(a: ScreenVertex, b: ScreenVertex, px: f32, py: f32) -> f32 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}

pub fn fill_triangle(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    a: ScreenVertex,
    b: ScreenVertex,
    c: ScreenVertex,
    color: Color,
) {
    let area = edge(a, b, c.x, c.y);
    if area == 0.0 {
        return;
    }

    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min(framebuffer.width() as f32 - 1.0) as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min(framebuffer.height() as f32 - 1.0) as i32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let weight_a = edge(b, c, px, py) / area;
            let weight_b = edge(c, a, px, py) / area;
            let weight_c = edge(a, b, px, py) / area;
            if weight_a < 0.0 || weight_b < 0.0 || weight_c < 0.0 {
                continue;
            }
            let depth = weight_a * a.depth + weight_b * b.depth + weight_c * c.depth;
            if depth_buffer.test_and_set(x, y, depth) {
                framebuffer.set_pixel(x, y, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel_at(framebuffer: &Framebuffer, x: usize, y: usize) -> &[u8] {
        let start = (y * framebuffer.width() + x) * 4;
        &framebuffer.pixels()[start..start + 3]
    }

    fn triangle_at(depth: f32) -> [ScreenVertex; 3] {
        [
            ScreenVertex::new(10.0, 10.0, depth),
            ScreenVertex::new(100.0, 10.0, depth),
            ScreenVertex::new(10.0, 100.0, depth),
        ]
    }

    #[test]
    fn nearest_triangle_wins_in_any_draw_order() {
        let red = Color::new(255, 0, 0);
        let green = Color::new(0, 255, 0);
        for order in [[(0.2, red), (0.8, green)], [(0.8, green), (0.2, red)]] {
            let mut framebuffer = Framebuffer::new();
            let mut depth_buffer = DepthBuffer::new();
            for (depth, color) in order {
                let [a, b, c] = triangle_at(depth);
                fill_triangle(&mut framebuffer, &mut depth_buffer, a, b, c, color);
            }
            assert_eq!(pixel_at(&framebuffer, 20, 20), &[255, 0, 0]);
        }
    }

    #[test]
    fn both_windings_fill_and_outside_stays_empty() {
        let white = Color::new(255, 255, 255);
        let [a, b, c] = triangle_at(0.5);
        let mut framebuffer = Framebuffer::new();
        let mut depth_buffer = DepthBuffer::new();
        fill_triangle(&mut framebuffer, &mut depth_buffer, a, c, b, white);
        assert_eq!(pixel_at(&framebuffer, 20, 20), &[255, 255, 255]);
        assert_eq!(pixel_at(&framebuffer, 90, 90), &[0, 0, 0]);
    }
}
