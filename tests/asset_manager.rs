use gc_proyecto_3::assets::material::{solid, ShaderKind};
use gc_proyecto_3::assets::AssetManager;
use gc_proyecto_3::engine::math::Vec3;
use std::fs;

const CUBE_OBJ: &str = "
o cubo
v -1.0 -1.0  1.0
v  1.0 -1.0  1.0
v  1.0  1.0  1.0
v -1.0  1.0  1.0
v -1.0 -1.0 -1.0
v  1.0 -1.0 -1.0
v  1.0  1.0 -1.0
v -1.0  1.0 -1.0
f 1 2 3 4
f 6 5 8 7
f 5 1 4 8
f 2 6 7 3
f 4 3 7 8
f 5 6 2 1
";

#[test]
fn loads_cube_from_text_with_twelve_triangles_and_outward_normals() {
    let mut assets = AssetManager::new();
    let handle = assets.load_mesh_from_source("cubo", CUBE_OBJ).unwrap();
    let mesh = assets.mesh(handle);

    assert_eq!(mesh.vertices.len(), 8);
    assert_eq!(mesh.triangle_count(), 12);
    for vertex in &mesh.vertices {
        assert!((vertex.normal.length() - 1.0).abs() < 1e-5);
        assert!(vertex.normal.dot(vertex.position) > 0.0);
    }
    for triangle in mesh.indices.chunks_exact(3) {
        let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
            .map(|index| mesh.vertices[index as usize].position);
        let center = (a + b + c) * (1.0 / 3.0);
        assert!((b - a).cross(c - a).dot(center) > 0.0);
    }
    assert!((assets.mesh_bounding_radius(handle) - 3.0_f32.sqrt()).abs() < 1e-5);
}

#[test]
fn shares_a_single_mesh_between_instances() {
    let mut assets = AssetManager::new();
    let first = assets.load_mesh_from_source("cubo", CUBE_OBJ).unwrap();
    let second = assets.load_mesh_from_source("cubo", CUBE_OBJ).unwrap();
    let built_once = assets.mesh_or_insert_with("cubo", || panic!("no debe reconstruirse"));

    assert_eq!(first, second);
    assert_eq!(first, built_once);
    assert_eq!(assets.mesh_count(), 1);
    assert!(std::ptr::eq(assets.mesh(first), assets.mesh(second)));
}

#[test]
fn loads_each_obj_file_only_once() {
    let path = std::env::temp_dir().join(format!("cubo_prueba_{}.obj", std::process::id()));
    fs::write(&path, CUBE_OBJ).unwrap();

    let mut assets = AssetManager::new();
    let first = assets.load_mesh(&path).unwrap();
    let second = assets.load_mesh(&path).unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(first, second);
    assert_eq!(assets.mesh_count(), 1);
    assert_eq!(assets.mesh(first).triangle_count(), 12);
}

#[test]
fn missing_file_returns_error_without_registering_mesh() {
    let mut assets = AssetManager::new();
    assert!(assets.load_mesh("no_existe/modelo.obj").is_err());
    assert_eq!(assets.mesh_count(), 0);
}

#[test]
fn materials_are_registered_by_name_and_editable() {
    let mut assets = AssetManager::new();
    let sand = Vec3::new(0.85, 0.76, 0.6);
    let rock = assets.add_material("roca", solid(sand, ShaderKind::Rocky));
    let again = assets.add_material("roca", solid(sand, ShaderKind::Lambert));

    assert_eq!(rock, again);
    assert_eq!(assets.material_count(), 1);
    assert_eq!(assets.material(rock).shader, ShaderKind::Lambert);

    assets.material_mut(rock).outline = false;
    assert!(!assets.material(rock).outline);
}
