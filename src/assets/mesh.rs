pub use crate::renderer::{Mesh, Vertex};

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
