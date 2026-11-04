use gc_proyecto_3::assets::obj_loader::{parse_obj, ObjError};
use gc_proyecto_3::engine::math::Vec3;

const PENTAGON_WITH_NORMALS: &str = "
o pentagono
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.3 1.0 0.0
v 0.5 1.6 0.0
v -0.3 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.5 1.0
vt 0.0 1.0
vn 0.0 0.0 2.0
s off
f 1/1/1 2/2/1 3/3/1 4/4/1 5/5/1
";

const QUAD_WITHOUT_NORMALS: &str = "
v -1.0 0.0 -1.0
v -1.0 0.0 1.0
v 1.0 0.0 1.0
v 1.0 0.0 -1.0
f -4 -3 -2 -1
";

#[test]
fn triangulates_polygons_as_fan() {
    let mesh = parse_obj(PENTAGON_WITH_NORMALS).unwrap();
    assert_eq!(mesh.vertices.len(), 5);
    assert_eq!(mesh.indices, vec![0, 1, 2, 0, 2, 3, 0, 3, 4]);
}

#[test]
fn keeps_file_normals_normalized_and_flips_uv() {
    let mesh = parse_obj(PENTAGON_WITH_NORMALS).unwrap();
    for vertex in &mesh.vertices {
        assert_eq!(vertex.normal, Vec3::new(0.0, 0.0, 1.0));
    }
    assert_eq!(mesh.vertices[2].uv.y, 0.0);
    assert_eq!(mesh.vertices[0].uv.y, 1.0);
}

#[test]
fn computes_normals_when_missing_and_supports_negative_indices() {
    let mesh = parse_obj(QUAD_WITHOUT_NORMALS).unwrap();
    assert_eq!(mesh.triangle_count(), 2);
    for vertex in &mesh.vertices {
        assert!((vertex.normal - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-5);
    }
}

#[test]
fn reports_line_of_invalid_data() {
    assert!(matches!(
        parse_obj("v 0 0 0\nv 1 0 0\nf 1 2 7\n"),
        Err(ObjError::InvalidIndex { line: 3 })
    ));
    assert!(matches!(
        parse_obj("v 0 x 0\n"),
        Err(ObjError::InvalidNumber { line: 1 })
    ));
    assert!(matches!(
        parse_obj("v 0 0 0\nv 1 0 0\nf 1 2\n"),
        Err(ObjError::MalformedFace { line: 3 })
    ));
    assert!(matches!(parse_obj("v 0 0 0\n"), Err(ObjError::NoFaces)));
}
