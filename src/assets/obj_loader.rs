use crate::engine::math::{Vec2, Vec3};
use crate::renderer::{Mesh, Vertex};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::str::SplitWhitespace;

#[derive(Debug)]
pub enum ObjError {
    Io(io::Error),
    InvalidNumber { line: usize },
    InvalidIndex { line: usize },
    MalformedFace { line: usize },
    NoFaces,
}

impl fmt::Display for ObjError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjError::Io(error) => write!(formatter, "no se pudo leer el archivo OBJ: {error}"),
            ObjError::InvalidNumber { line } => {
                write!(formatter, "numero invalido en la linea {line}")
            }
            ObjError::InvalidIndex { line } => {
                write!(formatter, "indice fuera de rango en la linea {line}")
            }
            ObjError::MalformedFace { line } => {
                write!(
                    formatter,
                    "cara con menos de tres vertices en la linea {line}"
                )
            }
            ObjError::NoFaces => write!(formatter, "el archivo OBJ no contiene caras"),
        }
    }
}

impl std::error::Error for ObjError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ObjError::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ObjError {
    fn from(error: io::Error) -> Self {
        ObjError::Io(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FaceCorner {
    position: usize,
    uv: Option<usize>,
    normal: Option<usize>,
}

#[derive(Default)]
struct ObjData {
    positions: Vec<Vec3>,
    uvs: Vec<Vec2>,
    normals: Vec<Vec3>,
}

#[derive(Default)]
struct MeshBuilder {
    mesh: Mesh,
    corner_indices: HashMap<FaceCorner, u32>,
    missing_normals: bool,
}

impl MeshBuilder {
    fn push_corner(&mut self, corner: FaceCorner, data: &ObjData) {
        let next_index = self.mesh.vertices.len() as u32;
        let index = *self.corner_indices.entry(corner).or_insert(next_index);
        if index == next_index {
            self.missing_normals |= corner.normal.is_none();
            self.mesh.vertices.push(Vertex {
                position: data.positions[corner.position],
                normal: corner
                    .normal
                    .map_or(Vec3::default(), |normal| data.normals[normal].normalize()),
                uv: corner.uv.map_or(Vec2::default(), |uv| data.uvs[uv]),
            });
        }
        self.mesh.indices.push(index);
    }

    fn push_polygon(&mut self, corners: &[FaceCorner], data: &ObjData) {
        for fan in 1..corners.len() - 1 {
            for corner in [corners[0], corners[fan], corners[fan + 1]] {
                self.push_corner(corner, data);
            }
        }
    }

    fn finish(mut self) -> Result<Mesh, ObjError> {
        if self.mesh.indices.is_empty() {
            return Err(ObjError::NoFaces);
        }
        if self.missing_normals {
            self.mesh.recalculate_normals();
        }
        Ok(self.mesh)
    }
}

pub fn load_obj(path: impl AsRef<Path>) -> Result<Mesh, ObjError> {
    let source = fs::read_to_string(path)?;
    parse_obj(&source)
}

pub fn parse_obj(source: &str) -> Result<Mesh, ObjError> {
    let mut data = ObjData::default();
    let mut builder = MeshBuilder::default();

    for (line_index, raw_line) in source.lines().enumerate() {
        let line = line_index + 1;
        let content = raw_line.split('#').next().unwrap_or_default();
        let mut tokens = content.split_whitespace();
        match tokens.next() {
            Some("v") => data.positions.push(parse_vec3(&mut tokens, line)?),
            Some("vn") => data.normals.push(parse_vec3(&mut tokens, line)?),
            Some("vt") => data.uvs.push(parse_uv(&mut tokens, line)?),
            Some("f") => {
                let corners = tokens
                    .map(|token| parse_corner(token, &data, line))
                    .collect::<Result<Vec<_>, _>>()?;
                if corners.len() < 3 {
                    return Err(ObjError::MalformedFace { line });
                }
                builder.push_polygon(&corners, &data);
            }
            _ => {}
        }
    }

    builder.finish()
}

fn parse_number(token: Option<&str>, line: usize) -> Result<f32, ObjError> {
    token
        .and_then(|text| text.parse::<f32>().ok())
        .filter(|value| value.is_finite())
        .ok_or(ObjError::InvalidNumber { line })
}

fn parse_vec3(tokens: &mut SplitWhitespace, line: usize) -> Result<Vec3, ObjError> {
    let x = parse_number(tokens.next(), line)?;
    let y = parse_number(tokens.next(), line)?;
    let z = parse_number(tokens.next(), line)?;
    Ok(Vec3::new(x, y, z))
}

fn parse_uv(tokens: &mut SplitWhitespace, line: usize) -> Result<Vec2, ObjError> {
    let u = parse_number(tokens.next(), line)?;
    let v = match tokens.next() {
        Some(text) => parse_number(Some(text), line)?,
        None => 0.0,
    };
    Ok(Vec2::new(u, 1.0 - v))
}

fn resolve_index(text: &str, count: usize, line: usize) -> Result<usize, ObjError> {
    let raw: i64 = text.parse().map_err(|_| ObjError::InvalidIndex { line })?;
    let count = count as i64;
    let resolved = if raw > 0 { raw - 1 } else { count + raw };
    if raw == 0 || resolved < 0 || resolved >= count {
        return Err(ObjError::InvalidIndex { line });
    }
    Ok(resolved as usize)
}

fn resolve_optional_index(
    text: Option<&str>,
    count: usize,
    line: usize,
) -> Result<Option<usize>, ObjError> {
    match text {
        Some(text) if !text.is_empty() => resolve_index(text, count, line).map(Some),
        _ => Ok(None),
    }
}

fn parse_corner(token: &str, data: &ObjData, line: usize) -> Result<FaceCorner, ObjError> {
    let mut parts = token.split('/');
    let position = resolve_index(parts.next().unwrap_or_default(), data.positions.len(), line)?;
    let uv = resolve_optional_index(parts.next(), data.uvs.len(), line)?;
    let normal = resolve_optional_index(parts.next(), data.normals.len(), line)?;
    Ok(FaceCorner {
        position,
        uv,
        normal,
    })
}
