use super::framebuffer::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};

pub struct DepthBuffer {
    width: usize,
    height: usize,
    depths: Vec<f32>,
}

impl DepthBuffer {
    pub fn new() -> Self {
        Self {
            width: FRAMEBUFFER_WIDTH,
            height: FRAMEBUFFER_HEIGHT,
            depths: vec![f32::INFINITY; FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        self.depths.fill(f32::INFINITY);
    }

    pub fn depth_at(&self, x: i32, y: i32) -> f32 {
        self.depths[y as usize * self.width + x as usize]
    }

    pub fn is_closer(&self, x: i32, y: i32, depth: f32) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < self.width
            && (y as usize) < self.height
            && depth < self.depth_at(x, y)
    }

    pub fn test_and_set(&mut self, x: i32, y: i32, depth: f32) -> bool {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return false;
        }
        let index = y as usize * self.width + x as usize;
        if depth < self.depths[index] {
            self.depths[index] = depth;
            true
        } else {
            false
        }
    }
}

impl Default for DepthBuffer {
    fn default() -> Self {
        Self::new()
    }
}
