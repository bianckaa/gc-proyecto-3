use super::material::{Material, MaterialHandle};
use super::mesh::{bounding_radius, Mesh, MeshHandle};
use super::obj_loader::{load_obj, parse_obj, ObjError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct MeshEntry {
    mesh: Mesh,
    bounding_radius: f32,
}

#[derive(Default)]
pub struct AssetManager {
    meshes: Vec<MeshEntry>,
    mesh_names: HashMap<String, MeshHandle>,
    materials: Vec<Material>,
    material_names: HashMap<String, MaterialHandle>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_mesh(&mut self, path: impl AsRef<Path>) -> Result<MeshHandle, ObjError> {
        let path = path.as_ref();
        let key = fs::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .into_owned();
        if let Some(handle) = self.find_mesh(&key) {
            return Ok(handle);
        }
        let mesh = load_obj(path)?;
        Ok(self.insert_mesh(key, mesh))
    }

    pub fn load_mesh_from_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> Result<MeshHandle, ObjError> {
        if let Some(handle) = self.find_mesh(name) {
            return Ok(handle);
        }
        let mesh = parse_obj(source)?;
        Ok(self.insert_mesh(name.to_owned(), mesh))
    }

    pub fn mesh_or_insert_with(&mut self, name: &str, build: impl FnOnce() -> Mesh) -> MeshHandle {
        match self.find_mesh(name) {
            Some(handle) => handle,
            None => self.insert_mesh(name.to_owned(), build()),
        }
    }

    pub fn find_mesh(&self, name: &str) -> Option<MeshHandle> {
        self.mesh_names.get(name).copied()
    }

    pub fn mesh(&self, handle: MeshHandle) -> &Mesh {
        &self.meshes[handle.index()].mesh
    }

    pub fn mesh_bounding_radius(&self, handle: MeshHandle) -> f32 {
        self.meshes[handle.index()].bounding_radius
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn add_material(&mut self, name: &str, material: Material) -> MaterialHandle {
        if let Some(handle) = self.find_material(name) {
            self.materials[handle.index()] = material;
            return handle;
        }
        let handle = MaterialHandle::new(self.materials.len());
        self.materials.push(material);
        self.material_names.insert(name.to_owned(), handle);
        handle
    }

    pub fn find_material(&self, name: &str) -> Option<MaterialHandle> {
        self.material_names.get(name).copied()
    }

    pub fn material(&self, handle: MaterialHandle) -> Material {
        self.materials[handle.index()]
    }

    pub fn material_mut(&mut self, handle: MaterialHandle) -> &mut Material {
        &mut self.materials[handle.index()]
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    fn insert_mesh(&mut self, key: String, mesh: Mesh) -> MeshHandle {
        let handle = MeshHandle::new(self.meshes.len());
        self.meshes.push(MeshEntry {
            bounding_radius: bounding_radius(&mesh),
            mesh,
        });
        self.mesh_names.insert(key, handle);
        handle
    }
}
