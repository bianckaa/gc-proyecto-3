use super::depth::DepthBuffer;
use super::framebuffer::{Color, Framebuffer};
use crate::engine::math::{Vec2, Vec3};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadedVertex {
    pub screen: ScreenVertex,
    pub inverse_w: f32,
    pub world_position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fragment {
    pub world_position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

fn edge(a: ScreenVertex, b: ScreenVertex, px: f32, py: f32) -> f32 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}

fn for_each_covered_pixel(
    width: usize,
    height: usize,
    a: ScreenVertex,
    b: ScreenVertex,
    c: ScreenVertex,
    mut visit: impl FnMut(i32, i32, [f32; 3]),
) {
    let area = edge(a, b, c.x, c.y);
    if area == 0.0 {
        return;
    }

    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min(width as f32 - 1.0) as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min(height as f32 - 1.0) as i32;

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
            visit(x, y, [weight_a, weight_b, weight_c]);
        }
    }
}

pub fn fill_triangle(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    a: ScreenVertex,
    b: ScreenVertex,
    c: ScreenVertex,
    color: Color,
) {
    let (width, height) = (framebuffer.width(), framebuffer.height());
    for_each_covered_pixel(width, height, a, b, c, |x, y, [wa, wb, wc]| {
        let depth = wa * a.depth + wb * b.depth + wc * c.depth;
        if depth_buffer.test_and_set(x, y, depth) {
            framebuffer.set_pixel(x, y, color);
        }
    });
}

pub fn fill_shaded_triangle(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    vertices: [ShadedVertex; 3],
    mut shade: impl FnMut(&Fragment) -> Color,
) {
    let [a, b, c] = vertices;
    let (width, height) = (framebuffer.width(), framebuffer.height());
    for_each_covered_pixel(width, height, a.screen, b.screen, c.screen, |x, y, weights| {
        let [wa, wb, wc] = weights;
        let depth = wa * a.screen.depth + wb * b.screen.depth + wc * c.screen.depth;
        if !depth_buffer.test_and_set(x, y, depth) {
            return;
        }
        let pa = wa * a.inverse_w;
        let pb = wb * b.inverse_w;
        let pc = wc * c.inverse_w;
        let total = pa + pb + pc;
        let (pa, pb, pc) = (pa / total, pb / total, pc / total);
        let fragment = Fragment {
            world_position: a.world_position * pa + b.world_position * pb + c.world_position * pc,
            normal: (a.normal * pa + b.normal * pb + c.normal * pc).normalize(),
            uv: a.uv * pa + b.uv * pb + c.uv * pc,
        };
        framebuffer.set_pixel(x, y, shade(&fragment));
    });
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
    fn uv_interpolation_favors_the_near_vertex() {
        let shaded = |x: f32, y: f32, inverse_w: f32, u: f32| ShadedVertex {
            screen: ScreenVertex::new(x, y, 0.5),
            inverse_w,
            world_position: Vec3::default(),
            normal: Vec3::new(0.0, 1.0, 0.0),
            uv: Vec2::new(u, 0.0),
        };
        let vertices = [
            shaded(0.0, 0.0, 1.0, 0.0),
            shaded(100.0, 0.0, 0.1, 1.0),
            shaded(0.0, 100.0, 1.0, 0.0),
        ];
        let mut framebuffer = Framebuffer::new();
        let mut depth_buffer = DepthBuffer::new();
        let mut sampled = Vec::new();
        fill_shaded_triangle(&mut framebuffer, &mut depth_buffer, vertices, |fragment| {
            sampled.push(*fragment);
            Color::BLACK
        });
        let expected = 0.0505 / 0.5455;
        assert!((sampled[50].uv.x - expected).abs() < 0.001);
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
