use super::depth::DepthBuffer;
use super::framebuffer::{Color, Framebuffer};
use super::lines::to_screen;
use super::rasterizer::fill_triangle;
use super::Mesh;
use crate::engine::math::Mat4;

pub fn draw_mesh(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    mesh: &Mesh,
    model: Mat4,
    view_projection: Mat4,
    color: Color,
) {
    let width = framebuffer.width() as f32;
    let height = framebuffer.height() as f32;
    let model_view_projection = view_projection * model;

    for triangle in mesh.indices.chunks_exact(3) {
        let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(|index| {
            model_view_projection * mesh.vertices[index as usize].position.extend(1.0)
        });
        if a.w <= 0.0 || b.w <= 0.0 || c.w <= 0.0 {
            continue;
        }
        fill_triangle(
            framebuffer,
            depth_buffer,
            to_screen(a, width, height),
            to_screen(b, width, height),
            to_screen(c, width, height),
            color,
        );
    }
}
