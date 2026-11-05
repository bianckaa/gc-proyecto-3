use super::transform::Transform;
use crate::assets::{AssetManager, LodHandle, MeshHandle};
use crate::renderer::{Material, RenderObject};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    Mesh(MeshHandle),
    Lod(LodHandle),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entity {
    pub model: Model,
    pub material: Material,
    pub transform: Transform,
    pub radius: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Scene {
    entities: Vec<Entity>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn push(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn render_objects<'a>(&self, assets: &'a AssetManager) -> Vec<RenderObject<'a>> {
        self.entities
            .iter()
            .map(|entity| {
                let (mesh, lod) = match entity.model {
                    Model::Mesh(handle) => (assets.mesh(handle), None),
                    Model::Lod(handle) => {
                        let group = assets.lod(handle);
                        (&group.levels[0], Some(group))
                    }
                };
                RenderObject {
                    mesh,
                    model_matrix: entity.transform.model_matrix(),
                    material: entity.material,
                    bounding_radius: entity.radius,
                    lod,
                }
            })
            .collect()
    }
}
