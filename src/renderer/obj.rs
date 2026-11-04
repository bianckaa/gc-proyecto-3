use super::{Mesh, Vertex};
use crate::engine::math::{Vec2, Vec3};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjError {
    line: usize,
    message: String,
}

impl ObjError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl Display for ObjError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "OBJ linea {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ObjError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct VertexKey {
    position: usize,
    uv: Option<usize>,
    normal: Option<usize>,
}

pub fn load_normalized(source: &str) -> Result<Mesh, ObjError> {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut mesh = Mesh::default();
    let mut vertices = HashMap::new();

    for (line_index, raw_line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let directive = fields.next().unwrap_or_default();
        match directive {
            "v" => positions.push(parse_vec3(fields, line_number)?),
            "vn" => normals.push(parse_vec3(fields, line_number)?.normalize()),
            "vt" => uvs.push(parse_vec2(fields, line_number)?),
            "f" => {
                let corners: Result<Vec<_>, _> = fields
                    .map(|field| {
                        parse_face_vertex(
                            field,
                            positions.len(),
                            uvs.len(),
                            normals.len(),
                            line_number,
                        )
                    })
                    .collect();
                let corners = corners?;
                if corners.len() < 3 {
                    return Err(ObjError::new(
                        line_number,
                        "una cara necesita al menos 3 vertices",
                    ));
                }
                for index in 1..corners.len() - 1 {
                    for key in [corners[0], corners[index], corners[index + 1]] {
                        let vertex_index = *vertices.entry(key).or_insert_with(|| {
                            let index = mesh.vertices.len() as u32;
                            mesh.vertices.push(Vertex {
                                position: positions[key.position],
                                normal: key.normal.map_or(Vec3::default(), |i| normals[i]),
                                uv: key.uv.map_or(Vec2::default(), |i| uvs[i]),
                            });
                            index
                        });
                        mesh.indices.push(vertex_index);
                    }
                }
            }
            _ => {}
        }
    }

    if mesh.indices.is_empty() {
        return Err(ObjError::new(0, "el archivo no contiene caras"));
    }
    if mesh
        .vertices
        .iter()
        .all(|vertex| vertex.normal == Vec3::default())
    {
        mesh.recalculate_normals();
    }
    normalize_to_unit_radius(&mut mesh)?;
    Ok(mesh)
}

fn parse_vec3<'a>(
    mut fields: impl Iterator<Item = &'a str>,
    line: usize,
) -> Result<Vec3, ObjError> {
    Ok(Vec3::new(
        parse_float(fields.next(), line)?,
        parse_float(fields.next(), line)?,
        parse_float(fields.next(), line)?,
    ))
}

fn parse_vec2<'a>(
    mut fields: impl Iterator<Item = &'a str>,
    line: usize,
) -> Result<Vec2, ObjError> {
    Ok(Vec2::new(
        parse_float(fields.next(), line)?,
        parse_float(fields.next(), line)?,
    ))
}

fn parse_float(field: Option<&str>, line: usize) -> Result<f32, ObjError> {
    field
        .ok_or_else(|| ObjError::new(line, "faltan componentes numericos"))?
        .parse::<f32>()
        .map_err(|_| ObjError::new(line, "numero invalido"))
}

fn parse_face_vertex(
    field: &str,
    position_count: usize,
    uv_count: usize,
    normal_count: usize,
    line: usize,
) -> Result<VertexKey, ObjError> {
    let mut indices = field.split('/');
    let position = resolve_index(indices.next(), position_count, line)?
        .ok_or_else(|| ObjError::new(line, "indice de posicion vacio"))?;
    let uv = resolve_index(indices.next(), uv_count, line)?;
    let normal = resolve_index(indices.next(), normal_count, line)?;
    if indices.next().is_some() {
        return Err(ObjError::new(line, "vertice de cara invalido"));
    }
    Ok(VertexKey {
        position,
        uv,
        normal,
    })
}

fn resolve_index(
    field: Option<&str>,
    count: usize,
    line: usize,
) -> Result<Option<usize>, ObjError> {
    let Some(field) = field.filter(|field| !field.is_empty()) else {
        return Ok(None);
    };
    let raw = field
        .parse::<isize>()
        .map_err(|_| ObjError::new(line, "indice invalido"))?;
    let resolved = if raw > 0 {
        raw - 1
    } else if raw < 0 {
        count as isize + raw
    } else {
        return Err(ObjError::new(line, "los indices OBJ no pueden ser cero"));
    };
    if resolved < 0 || resolved as usize >= count {
        return Err(ObjError::new(line, "indice fuera de rango"));
    }
    Ok(Some(resolved as usize))
}

fn normalize_to_unit_radius(mesh: &mut Mesh) -> Result<(), ObjError> {
    let first = mesh
        .vertices
        .first()
        .ok_or_else(|| ObjError::new(0, "el archivo no contiene vertices utilizados"))?
        .position;
    let mut minimum = first;
    let mut maximum = first;
    for vertex in &mesh.vertices {
        minimum.x = minimum.x.min(vertex.position.x);
        minimum.y = minimum.y.min(vertex.position.y);
        minimum.z = minimum.z.min(vertex.position.z);
        maximum.x = maximum.x.max(vertex.position.x);
        maximum.y = maximum.y.max(vertex.position.y);
        maximum.z = maximum.z.max(vertex.position.z);
    }
    let center = (minimum + maximum) * 0.5;
    let radius = mesh
        .vertices
        .iter()
        .map(|vertex| (vertex.position - center).length())
        .fold(0.0_f32, f32::max);
    if radius <= f32::EPSILON {
        return Err(ObjError::new(0, "el modelo tiene radio cero"));
    }
    for vertex in &mut mesh.vertices {
        vertex.position = (vertex.position - center) * (1.0 / radius);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_face_attributes_and_normalizes_the_model() {
        let source = "\
v -2 0 0
v 2 0 0
v 0 2 0
vt 0 0
vt 1 0
vt 0.5 1
vn 0 0 1
f 1/1/1 2/2/1 3/3/1
";
        let mesh = load_normalized(source).unwrap();
        assert_eq!(mesh.triangle_count(), 1);
        assert_eq!(mesh.vertices.len(), 3);
        assert!(mesh
            .vertices
            .iter()
            .all(|vertex| vertex.position.length() <= 1.0001));
        assert_eq!(mesh.vertices[0].normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn preserves_split_normals_for_flat_shading() {
        let source = "\
v 0 0 0
v 1 0 0
v 0 1 0
v 0 0 1
vn 0 0 1
vn 1 0 0
f 1//1 2//1 3//1
f 1//2 4//2 2//2
";
        let mesh = load_normalized(source).unwrap();
        assert_eq!(mesh.triangle_count(), 2);
        assert_eq!(mesh.vertices.len(), 6);
    }

    #[test]
    fn accepts_negative_indices_and_polygon_fans() {
        let source = "\
v -1 -1 0
v 1 -1 0
v 1 1 0
v -1 1 0
f -4 -3 -2 -1
";
        let mesh = load_normalized(source).unwrap();
        assert_eq!(mesh.triangle_count(), 2);
    }
}
