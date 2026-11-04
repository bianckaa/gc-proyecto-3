use gc_proyecto_3::assets::mesh::placeholder_ship;
use gc_proyecto_3::assets::AssetManager;
use std::fs;
use std::path::PathBuf;

const TRIANGLE_OBJ: &str = "
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
";

fn temporary_obj(name: &str, source: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{name}_{}.obj", std::process::id()));
    fs::write(&path, source).unwrap();
    path
}

#[test]
fn placeholder_ship_is_a_flat_rhombus_pointing_forward() {
    let mesh = placeholder_ship();
    assert_eq!(mesh.triangle_count(), 8);

    let min_z = mesh
        .vertices
        .iter()
        .map(|v| v.position.z)
        .fold(f32::MAX, f32::min);
    let max_z = mesh
        .vertices
        .iter()
        .map(|v| v.position.z)
        .fold(f32::MIN, f32::max);
    let span = mesh
        .vertices
        .iter()
        .map(|v| v.position.x.abs())
        .fold(0.0, f32::max);
    let thickness = mesh
        .vertices
        .iter()
        .map(|v| v.position.y.abs())
        .fold(0.0, f32::max);
    assert!(-min_z > max_z);
    assert!(thickness * 4.0 < span);

    for triangle in mesh.indices.chunks_exact(3) {
        let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
            .map(|index| mesh.vertices[index as usize].position);
        let center = (a + b + c) * (1.0 / 3.0);
        assert!((b - a).cross(c - a).dot(center) > 0.0);
    }
}

#[test]
fn missing_model_falls_back_to_placeholder_built_once() {
    let mut assets = AssetManager::new();
    let missing = std::env::temp_dir().join("no_existe_nave.obj");
    let first = assets.load_mesh_or_else(&missing, "temporal", placeholder_ship);
    let second = assets.load_mesh_or_else(&missing, "temporal", || panic!("no debe reconstruirse"));

    assert_eq!(first, second);
    assert_eq!(assets.mesh_count(), 1);
    assert_eq!(assets.mesh(first).triangle_count(), 8);
}

#[test]
fn existing_model_replaces_placeholder() {
    let path = temporary_obj("nave_prueba", TRIANGLE_OBJ);
    let mut assets = AssetManager::new();
    let handle = assets.load_mesh_or_else(&path, "temporal", || panic!("debe usar el archivo"));
    fs::remove_file(&path).unwrap();

    assert_eq!(assets.mesh(handle).triangle_count(), 1);
    assert!(assets.find_mesh("temporal").is_none());
}

#[test]
fn invalid_model_falls_back_to_placeholder() {
    let path = temporary_obj("nave_invalida", "v 0 0 0\n");
    let mut assets = AssetManager::new();
    let handle = assets.load_mesh_or_else(&path, "temporal", placeholder_ship);
    fs::remove_file(&path).unwrap();

    assert_eq!(Some(handle), assets.find_mesh("temporal"));
    assert_eq!(assets.mesh(handle).triangle_count(), 8);
}
