use gc_proyecto_3::assets::material::{solid, ShaderKind};
use gc_proyecto_3::assets::mesh::placeholder_ship;
use gc_proyecto_3::assets::AssetManager;
use gc_proyecto_3::engine::math::Vec3;
use gc_proyecto_3::engine::scene::{Entity, Model, Scene};
use gc_proyecto_3::engine::transform::{basis_rotation, rotate_vector, Transform};

fn close(a: Vec3, b: Vec3) -> bool {
    (a - b).length() < 1e-4
}

#[test]
fn basis_rotation_points_local_forward_and_up_where_asked() {
    let forward = Vec3::new(1.0, 0.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    let rotation = basis_rotation(forward, up);
    assert!(close(
        rotate_vector(rotation, Vec3::new(0.0, 0.0, -1.0)),
        forward
    ));
    assert!(close(rotate_vector(rotation, Vec3::new(0.0, 1.0, 0.0)), up));
}

#[test]
fn transform_applies_scale_then_rotation_then_translation() {
    let transform = Transform::new(Vec3::new(10.0, 0.0, 0.0))
        .with_rotation(basis_rotation(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ))
        .with_uniform_scale(2.0);
    let point = transform.transform_point(Vec3::new(0.0, 0.0, -1.0));
    assert!(close(point, Vec3::new(12.0, 0.0, 0.0)));
    assert_eq!(transform.max_scale(), 2.0);
}

#[test]
fn scene_instances_share_the_same_mesh() {
    let mut assets = AssetManager::new();
    let handle = assets.mesh_or_insert_with("nave", placeholder_ship);
    let mut scene = Scene::new();
    for x in 0..3 {
        scene.push(Entity {
            model: Model::Mesh(handle),
            material: solid(Vec3::new(1.0, 1.0, 1.0), ShaderKind::Lambert),
            transform: Transform::new(Vec3::new(x as f32, 0.0, 0.0)),
            radius: 1.0,
        });
    }
    let objects = scene.render_objects(&assets);
    assert_eq!(objects.len(), 3);
    assert!(std::ptr::eq(objects[0].mesh, objects[2].mesh));
    assert_eq!(objects[2].model_matrix.rows[0][3], 2.0);
}
