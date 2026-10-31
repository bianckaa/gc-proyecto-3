pub mod depth;
pub mod framebuffer;
pub mod lines;
pub mod pipeline;
pub mod rasterizer;

use crate::engine::math::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
