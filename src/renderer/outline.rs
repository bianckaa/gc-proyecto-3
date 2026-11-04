use super::depth::DepthBuffer;
use super::framebuffer::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};

pub const NO_OBJECT: u32 = 0;
const RELATIVE_DEPTH_JUMP: f32 = 0.25;
const NEIGHBORS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

pub struct ObjectBuffer {
    ids: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlinePixel {
    pub x: i32,
    pub y: i32,
    pub object_id: u32,
}

fn in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && (x as usize) < FRAMEBUFFER_WIDTH && (y as usize) < FRAMEBUFFER_HEIGHT
}

fn linear_depth(depth: f32, near: f32, far: f32) -> f32 {
    if depth.is_finite() {
        2.0 * near * far / (far + near - depth * (far - near))
    } else {
        f32::INFINITY
    }
}

impl ObjectBuffer {
    pub fn new() -> Self {
        Self {
            ids: vec![NO_OBJECT; FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        self.ids.fill(NO_OBJECT);
    }

    pub fn set(&mut self, x: i32, y: i32, object_id: u32) {
        if in_bounds(x, y) {
            self.ids[y as usize * FRAMEBUFFER_WIDTH + x as usize] = object_id;
        }
    }

    pub(crate) fn get(&self, x: i32, y: i32) -> u32 {
        self.ids[y as usize * FRAMEBUFFER_WIDTH + x as usize]
    }
}

impl Default for ObjectBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn find_outline_pixels(
    objects: &ObjectBuffer,
    depth_buffer: &DepthBuffer,
    near: f32,
    far: f32,
    outline: &mut Vec<OutlinePixel>,
) {
    outline.clear();
    let depth_at = |x: i32, y: i32| linear_depth(depth_buffer.depth_at(x, y), near, far);

    for y in 0..FRAMEBUFFER_HEIGHT as i32 {
        for x in 0..FRAMEBUFFER_WIDTH as i32 {
            let object_id = objects.get(x, y);
            if object_id == NO_OBJECT {
                continue;
            }
            let depth = depth_at(x, y);
            let is_edge = NEIGHBORS.iter().any(|&(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                if !in_bounds(nx, ny) {
                    return false;
                }
                let neighbor_depth = depth_at(nx, ny);
                if objects.get(nx, ny) == object_id {
                    neighbor_depth - depth > depth * RELATIVE_DEPTH_JUMP
                } else {
                    depth <= neighbor_depth
                }
            });
            if is_edge {
                outline.push(OutlinePixel { x, y, object_id });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_marks_its_border_but_not_its_interior() {
        let mut objects = ObjectBuffer::new();
        let mut depth_buffer = DepthBuffer::new();
        for y in 10..15 {
            for x in 10..15 {
                objects.set(x, y, 1);
                depth_buffer.test_and_set(x, y, 0.5);
            }
        }
        let mut outline = Vec::new();
        find_outline_pixels(&objects, &depth_buffer, 0.1, 100.0, &mut outline);
        assert_eq!(outline.len(), 16);
        assert!(!outline.iter().any(|pixel| pixel.x == 12 && pixel.y == 12));
    }
}
