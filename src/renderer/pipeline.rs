use super::clipping::{clip_against_near, ClipVertex};
use super::depth::DepthBuffer;
use super::framebuffer::{Color, Framebuffer};
use super::lines::to_screen;
use super::rasterizer::fill_triangle;
use super::{Mesh, Vertex};
use crate::engine::math::Mat4;

fn to_clip_space(vertex: &Vertex, model: Mat4, view_projection: Mat4) -> ClipVertex {
    let world = model * vertex.position.extend(1.0);
    ClipVertex {
        position: view_projection * world,
        world_position: world.truncate(),
        normal: model.transform_normal(vertex.normal),
        uv: vertex.uv,
    }
}

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

    for triangle in mesh.indices.chunks_exact(3) {
        let clip_triangle = [triangle[0], triangle[1], triangle[2]]
            .map(|index| to_clip_space(&mesh.vertices[index as usize], model, view_projection));

        for [a, b, c] in clip_against_near(clip_triangle).triangles() {
            let [a, b, c] = [a, b, c].map(|vertex| to_screen(vertex.position, width, height));
            fill_triangle(framebuffer, depth_buffer, a, b, c, color);
        }
    }
}
