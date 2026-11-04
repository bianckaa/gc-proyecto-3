use crate::engine::math::{Vec2, Vec3};
pub use crate::renderer::{Mesh, Vertex};

const PLACEHOLDER_SHIP_NOSE: Vec3 = Vec3::new(0.0, 0.0, -1.5);
const PLACEHOLDER_SHIP_TAIL: Vec3 = Vec3::new(0.0, 0.0, 0.6);
const PLACEHOLDER_SHIP_HALF_SPAN: f32 = 1.1;
const PLACEHOLDER_SHIP_HALF_THICKNESS: f32 = 0.12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(usize);

impl MeshHandle {
    pub(super) fn new(index: usize) -> Self {
        Self(index)
    }

    pub(super) fn index(self) -> usize {
        self.0
    }
}

pub fn bounding_radius(mesh: &Mesh) -> f32 {
    mesh.vertices
        .iter()
        .map(|vertex| vertex.position.length())
        .fold(0.0, f32::max)
}

pub fn placeholder_ship() -> Mesh {
    let left = Vec3::new(-PLACEHOLDER_SHIP_HALF_SPAN, 0.0, 0.0);
    let right = Vec3::new(PLACEHOLDER_SHIP_HALF_SPAN, 0.0, 0.0);
    let top = Vec3::new(0.0, PLACEHOLDER_SHIP_HALF_THICKNESS, 0.0);
    let bottom = Vec3::new(0.0, -PLACEHOLDER_SHIP_HALF_THICKNESS, 0.0);
    let mut mesh = Mesh::default();
    for side in [left, right] {
        for face in [top, bottom] {
            push_outward_triangle(&mut mesh, [PLACEHOLDER_SHIP_NOSE, side, face]);
            push_outward_triangle(&mut mesh, [PLACEHOLDER_SHIP_TAIL, side, face]);
        }
    }
    mesh
}

fn push_outward_triangle(mesh: &mut Mesh, corners: [Vec3; 3]) {
    let [a, b, c] = corners;
    let normal = (b - a).cross(c - a).normalize();
    let center = (a + b + c) * (1.0 / 3.0);
    let (ordered, normal) = if normal.dot(center) >= 0.0 {
        ([a, b, c], normal)
    } else {
        ([a, c, b], -normal)
    };
    for position in ordered {
        mesh.indices.push(mesh.vertices.len() as u32);
        mesh.vertices.push(Vertex {
            position,
            normal,
            uv: Vec2::default(),
        });
    }
}
