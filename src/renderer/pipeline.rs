use super::clipping::{clip_against_near, ClipVertex};
use super::culling::is_back_facing;
use super::depth::DepthBuffer;
use super::framebuffer::Framebuffer;
use super::lines::to_screen;
use super::rasterizer::{fill_shaded_triangle, Fragment, FragmentOutput, ShadedVertex};
use super::{Mesh, Vertex};
use crate::engine::math::{Mat4, Vec3};

const MIN_UV_AREA: f32 = 1e-8;

fn triangle_tangent(vertices: [&Vertex; 3]) -> Vec3 {
    let [a, b, c] = vertices;
    let (edge_b, edge_c) = (b.position - a.position, c.position - a.position);
    let (uv_b, uv_c) = (b.uv - a.uv, c.uv - a.uv);
    let area = uv_b.x * uv_c.y - uv_c.x * uv_b.y;
    if area.abs() < MIN_UV_AREA {
        return Vec3::default();
    }
    (edge_b * uv_c.y - edge_c * uv_b.y) * (1.0 / area)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DrawStats {
    pub processed: usize,
    pub drawn: usize,
}

fn to_clip_space(vertex: &Vertex, tangent: Vec3, model: Mat4, view_projection: Mat4) -> ClipVertex {
    let world = model * vertex.position.extend(1.0);
    ClipVertex {
        position: view_projection * world,
        local_position: vertex.position,
        world_position: world.truncate(),
        normal: model.transform_normal(vertex.normal),
        tangent: (model * tangent.extend(0.0)).truncate(),
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
        tangent: vertex.tangent,
        uv: vertex.uv,
    }
}

pub fn draw_mesh(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    mesh: &Mesh,
    model: Mat4,
    view_projection: Mat4,
    mut shade: impl FnMut(&Fragment) -> Option<FragmentOutput>,
) -> DrawStats {
    let width = framebuffer.width() as f32;
    let height = framebuffer.height() as f32;
    let mut stats = DrawStats {
        processed: mesh.triangle_count(),
        drawn: 0,
    };

    for triangle in mesh.indices.chunks_exact(3) {
        let corners = [triangle[0], triangle[1], triangle[2]].map(|index| &mesh.vertices[index as usize]);
        let tangent = triangle_tangent(corners);
        let clip_triangle =
            corners.map(|vertex| to_clip_space(vertex, tangent, model, view_projection));

        for clipped in clip_against_near(clip_triangle).triangles() {
            let shaded = clipped.map(|vertex| to_shaded(vertex, width, height));
            if !is_back_facing(shaded[0].screen, shaded[1].screen, shaded[2].screen) {
                fill_shaded_triangle(framebuffer, depth_buffer, shaded, &mut shade);
                stats.drawn += 1;
            }
        }
    }
    stats
}
