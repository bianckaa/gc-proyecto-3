use super::material::{Material, MaterialHandle};
use super::mesh::{
    bounding_radius, normalize_to_unit_radius, placeholder_ship, turned_around, LodHandle, Mesh,
    MeshHandle,
};
use super::obj_loader::{load_obj, parse_obj, ObjError};
use crate::renderer::LodGroup;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const MODELS_DIRECTORY: &str = "assets/models";
pub const SHIP_MODEL_FILE: &str = "ETA_2.obj";
pub const SHIP_MODEL_PATH: &str = "assets/models/ETA_2.obj";
const PLACEHOLDER_SHIP_NAME: &str = "nave_temporal";
const MODEL_KEY_PREFIX: &str = "modelo:";

pub fn model_path(file_name: &str) -> Option<PathBuf> {
    let relative = Path::new(MODELS_DIRECTORY).join(file_name);
    let mut candidates = vec![relative.clone()];
    if let Ok(executable) = std::env::current_exe() {
        candidates.extend(
            executable
                .ancestors()
                .skip(1)
                .map(|folder| folder.join(&relative)),
        );
    }
    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join(&relative));
    candidates.into_iter().find(|path| path.is_file())
}

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
    lods: Vec<LodGroup>,
    lod_names: HashMap<String, LodHandle>,
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

    pub fn load_mesh_or_else(
        &mut self,
        path: impl AsRef<Path>,
        fallback_name: &str,
        build_fallback: impl FnOnce() -> Mesh,
    ) -> MeshHandle {
        let path = path.as_ref();
        if path.is_file() {
            match self.load_mesh(path) {
                Ok(handle) => return handle,
                Err(error) => eprintln!(
                    "no se pudo cargar {}: {error}; se usa la malla temporal",
                    path.display()
                ),
            }
        }
        self.mesh_or_insert_with(fallback_name, build_fallback)
    }

    pub fn load_model_or_else(
        &mut self,
        file_name: &str,
        build_fallback: impl FnOnce() -> Mesh,
    ) -> MeshHandle {
        let key = format!("{MODEL_KEY_PREFIX}{file_name}");
        if let Some(handle) = self.find_mesh(&key) {
            return handle;
        }
        let loaded = match model_path(file_name) {
            Some(path) => load_obj(&path).map_err(|error| {
                eprintln!(
                    "no se pudo cargar {}: {error}; se usa la malla temporal",
                    path.display()
                )
            }),
            None => {
                eprintln!(
                    "no se encontro {MODELS_DIRECTORY}/{file_name}; se usa la malla temporal"
                );
                Err(())
            }
        };
        let mut mesh = loaded.unwrap_or_else(|_| build_fallback());
        normalize_to_unit_radius(&mut mesh);
        self.insert_mesh(key, mesh)
    }

    pub fn optional_model_or_else(
        &mut self,
        file_name: &str,
        build_fallback: impl FnOnce() -> Mesh,
    ) -> MeshHandle {
        if model_path(file_name).is_some() {
            return self.load_model_or_else(file_name, build_fallback);
        }
        self.mesh_or_insert_with(file_name, || {
            let mut mesh = build_fallback();
            normalize_to_unit_radius(&mut mesh);
            mesh
        })
    }

    pub fn load_ship(&mut self) -> MeshHandle {
        if model_path(SHIP_MODEL_FILE).is_none() {
            return self.mesh_or_insert_with(PLACEHOLDER_SHIP_NAME, || {
                let mut mesh = turned_around(placeholder_ship());
                normalize_to_unit_radius(&mut mesh);
                mesh
            });
        }
        self.load_model_or_else(SHIP_MODEL_FILE, || turned_around(placeholder_ship()))
    }

    pub fn lod_or_insert_with(
        &mut self,
        name: &str,
        build: impl FnMut(usize) -> Mesh,
    ) -> LodHandle {
        if let Some(&handle) = self.lod_names.get(name) {
            return handle;
        }
        let handle = LodHandle::new(self.lods.len());
        self.lods.push(LodGroup::new(build));
        self.lod_names.insert(name.to_owned(), handle);
        handle
    }

    pub fn lod(&self, handle: LodHandle) -> &LodGroup {
        &self.lods[handle.index()]
    }

    pub fn lod_bounding_radius(&self, handle: LodHandle) -> f32 {
        bounding_radius(&self.lods[handle.index()].levels[0])
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
