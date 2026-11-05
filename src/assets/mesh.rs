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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LodHandle(usize);

impl LodHandle {
    pub(super) fn new(index: usize) -> Self {
        Self(index)
    }

    pub(super) fn index(self) -> usize {
        self.0
    }
}

pub fn normalize_to_unit_radius(mesh: &mut Mesh) {
    let Some(first) = mesh.vertices.first().map(|vertex| vertex.position) else {
        return;
    };
    let (mut minimum, mut maximum) = (first, first);
    for vertex in &mesh.vertices {
        let p = vertex.position;
        minimum = Vec3::new(minimum.x.min(p.x), minimum.y.min(p.y), minimum.z.min(p.z));
        maximum = Vec3::new(maximum.x.max(p.x), maximum.y.max(p.y), maximum.z.max(p.z));
    }
    let center = (minimum + maximum) * 0.5;
    let radius = mesh
        .vertices
        .iter()
        .map(|vertex| (vertex.position - center).length())
        .fold(0.0, f32::max);
    if radius <= f32::EPSILON {
        return;
    }
    for vertex in &mut mesh.vertices {
        vertex.position = (vertex.position - center) * (1.0 / radius);
    }
}

pub fn turned_around(mut mesh: Mesh) -> Mesh {
    for vertex in &mut mesh.vertices {
        vertex.position = Vec3::new(-vertex.position.x, vertex.position.y, -vertex.position.z);
        vertex.normal = Vec3::new(-vertex.normal.x, vertex.normal.y, -vertex.normal.z);
    }
    mesh
}

pub fn crystal() -> Mesh {
    let tip = Vec3::new(0.0, 1.0, 0.0);
    let base = Vec3::new(0.0, -0.25, 0.0);
    let ring = [
        Vec3::new(0.3, 0.1, 0.0),
        Vec3::new(0.0, 0.1, 0.22),
        Vec3::new(-0.3, 0.1, 0.0),
        Vec3::new(0.0, 0.1, -0.22),
    ];
    let mut mesh = Mesh::default();
    for index in 0..ring.len() {
        let next = ring[(index + 1) % ring.len()];
        push_outward_triangle(&mut mesh, [tip, ring[index], next]);
        push_outward_triangle(&mut mesh, [base, ring[index], next]);
    }
    mesh
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
