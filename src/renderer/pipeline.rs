use super::clipping::{clip_against_near, ClipVertex};
use super::culling::is_back_facing;
use super::depth::DepthBuffer;
use super::framebuffer::{Color, Framebuffer};
use super::lines::to_screen;
use super::rasterizer::{fill_shaded_triangle, Fragment, ShadedVertex};
use super::{Mesh, Vertex};
use crate::engine::math::Mat4;

fn to_clip_space(vertex: &Vertex, model: Mat4, view_projection: Mat4) -> ClipVertex {
    let world = model * vertex.position.extend(1.0);
    ClipVertex {
        position: view_projection * world,
        local_position: vertex.position,
        world_position: world.truncate(),
        normal: model.transform_normal(vertex.normal),
        uv: vertex.uv,
    }
}

fn to_shaded(vertex: ClipVertex, width: f32, height: f32) -> ShadedVertex {
    ShadedVertex {
        screen: to_screen(vertex.position, width, height),
        inverse_w: 1.0 / vertex.position.w,
        local_position: vertex.local_position,
        world_position: vertex.world_position,
        normal: vertex.normal,
        uv: vertex.uv,
    }
}

pub fn draw_mesh(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    mesh: &Mesh,
    model: Mat4,
    view_projection: Mat4,
    mut shade: impl FnMut(&Fragment) -> Color,
) {
    let width = framebuffer.width() as f32;
    let height = framebuffer.height() as f32;

    for triangle in mesh.indices.chunks_exact(3) {
        let clip_triangle = [triangle[0], triangle[1], triangle[2]]
            .map(|index| to_clip_space(&mesh.vertices[index as usize], model, view_projection));

        for clipped in clip_against_near(clip_triangle).triangles() {
            let shaded = clipped.map(|vertex| to_shaded(vertex, width, height));
            if !is_back_facing(shaded[0].screen, shaded[1].screen, shaded[2].screen) {
                fill_shaded_triangle(framebuffer, depth_buffer, shaded, &mut shade);
            }
        }
    }
}
