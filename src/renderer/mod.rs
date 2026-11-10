pub mod black_hole;
pub mod clipping;
pub mod culling;
pub mod depth;
pub mod dithering;
pub mod framebuffer;
pub mod heat_haze;
pub mod lighting;
pub mod lines;
pub mod normal_map;
pub mod opening_crawl;
pub mod obj;
pub mod outline;
pub mod pipeline;
pub mod pixel_font;
pub mod rasterizer;
pub mod shaders;
pub mod skybox;
pub mod warp_effect;

use crate::engine::camera::Camera;
use crate::engine::math::{Mat4, Vec2, Vec3};
use crate::debug::metrics::Metrics;
use black_hole::BlackHoleEffect;
use culling::Frustum;
use depth::DepthBuffer;
use dithering::{dither, dither_levels, screen_door_visible};
use framebuffer::{Color, Framebuffer};
use heat_haze::HeatHaze;
use lighting::{realistic_color, PointLight};
use normal_map::NormalMap;
use outline::{find_outline_pixels, ObjectBuffer, OutlinePixel};
use rasterizer::FragmentOutput;
use shaders::{fbm, shade, ShadeInput, ShaderKind};
use skybox::{SeededRandom, Skybox};
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use warp_effect::WarpEffect;

const DEFAULT_AMBIENT: f32 = 0.12;
const CROSSHAIR_RADIUS: i32 = 4;
const REALISTIC_DITHER_LEVELS: u32 = 6;
const STAR_SEED: u32 = 20_260_001;
const STAR_COUNT: usize = 1800;
const NORMAL_MAP_SEED: u32 = 20_260_002;
const LOD_HIGH_DISTANCE: f32 = 12.0;
const LOD_MEDIUM_DISTANCE: f32 = 40.0;
const CRATER_RIM_END: f32 = 1.4;
const CRATER_RIM_HEIGHT: f32 = 0.3;
const GAS_PLANET_OPACITY: f32 = 0.76;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
    pub base_color: Vec3,
    pub emissive: Vec3,
    pub shader: ShaderKind,
    pub outline: bool,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderObject<'a> {
    pub mesh: &'a Mesh,
    pub model_matrix: Mat4,
    pub material: Material,
    pub bounding_radius: f32,
    pub lod: Option<&'a LodGroup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LodGroup {
    pub levels: [Mesh; 3],
}

impl LodGroup {
    pub fn new(mut build: impl FnMut(usize) -> Mesh) -> Self {
        Self {
            levels: [build(0), build(1), build(2)],
        }
    }

    pub fn select(&self, relative_distance: f32) -> &Mesh {
        if relative_distance < LOD_HIGH_DISTANCE {
            &self.levels[0]
        } else if relative_distance < LOD_MEDIUM_DISTANCE {
            &self.levels[1]
        } else {
            &self.levels[2]
        }
    }
}

fn effective_opacity(material: &Material) -> f32 {
    let shader_opacity = if material.shader == ShaderKind::GasPlanet {
        GAS_PLANET_OPACITY
    } else {
        1.0
    };
    material.opacity.clamp(0.0, 1.0) * shader_opacity
}

struct QueuedLine {
    start: Vec3,
    end: Vec3,
    color: Color,
    dashed: bool,
}

pub struct Renderer {
    pub light: PointLight,
    pub dithering_enabled: bool,
    pub heat_haze_enabled: bool,
    pub outline_enabled: bool,
    pub culling_enabled: bool,
    pub lod_enabled: bool,
    pub normal_mapping_enabled: bool,
    pub nebula_enabled: bool,
    pub warp: WarpEffect,
    pub black_hole: BlackHoleEffect,
    pub metrics: Metrics,
    framebuffer: Framebuffer,
    depth_buffer: DepthBuffer,
    object_buffer: ObjectBuffer,
    outline_pixels: Vec<OutlinePixel>,
    heat_haze: HeatHaze,
    normal_map: NormalMap,
    skybox: Skybox,
    queued_lines: Vec<QueuedLine>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            light: PointLight::new(Vec3::default(), DEFAULT_AMBIENT),
            dithering_enabled: true,
            heat_haze_enabled: false,
            outline_enabled: true,
            culling_enabled: true,
            lod_enabled: true,
            normal_mapping_enabled: true,
            nebula_enabled: true,
            warp: WarpEffect::new(),
            black_hole: BlackHoleEffect::new(),
            metrics: Metrics::new(),
            framebuffer: Framebuffer::new(),
            depth_buffer: DepthBuffer::new(),
            object_buffer: ObjectBuffer::new(),
            outline_pixels: Vec::new(),
            heat_haze: HeatHaze::new(),
            normal_map: NormalMap::procedural(NORMAL_MAP_SEED),
            skybox: Skybox::new(STAR_SEED, STAR_COUNT),
            queued_lines: Vec::new(),
        }
    }

    pub fn pixels(&self) -> &[u8] {
        self.framebuffer.pixels()
    }

    pub fn fade_to_white(&mut self, amount: f32) {
        self.framebuffer.fade_to_white(amount);
    }

    pub fn tint_screen(&mut self, color: Color, amount: f32) {
        self.framebuffer.tint(color, amount);
    }

    pub fn framebuffer_mut(&mut self) -> &mut Framebuffer {
        &mut self.framebuffer
    }

    pub fn render(&mut self, objects: &[RenderObject], camera: &Camera, time: f32) {
        self.framebuffer.clear(Color::BLACK);
        self.depth_buffer.clear();
        if self.nebula_enabled {
            self.skybox.draw_nebula(&mut self.framebuffer, camera);
        }
        self.skybox.draw(&mut self.framebuffer, camera, time);
        let view_projection = camera.view_projection_matrix();
        let light = self.light;
        let dithering_enabled = self.dithering_enabled;
        let eye = camera.position;
        let normal_mapping_enabled = self.normal_mapping_enabled;
        let normal_map = Some(&self.normal_map);

        self.object_buffer.clear();
        let object_buffer = &mut self.object_buffer;

        let frustum = Frustum::from_view_projection(view_projection);
        let mut drawn_triangles = 0;
        let mut processed_triangles = 0;
        let mut culled_objects = 0;

        for (index, object) in objects.iter().enumerate() {
            let model = object.model_matrix.rows;
            let center = Vec3::new(model[0][3], model[1][3], model[2][3]);
            if self.culling_enabled && !frustum.contains_sphere(center, object.bounding_radius) {
                culled_objects += 1;
                continue;
            }
            let distance = (center - camera.position).length();
            let relative_distance = distance / object.bounding_radius.max(f32::EPSILON);
            let mesh = object
                .lod
                .filter(|_| self.lod_enabled)
                .map_or(object.mesh, |group| group.select(relative_distance));
            let material = object.material;
            let opacity = effective_opacity(&material);
            if opacity <= 0.0 {
                continue;
            }
            let object_id = index as u32 + 1;
            let stats = pipeline::draw_mesh(
                &mut self.framebuffer,
                &mut self.depth_buffer,
                mesh,
                object.model_matrix,
                view_projection,
                |fragment| {
                    if dithering_enabled
                        && opacity < 1.0
                        && !screen_door_visible(opacity, fragment.x, fragment.y)
                    {
                        return None;
                    }
                    object_buffer.set(fragment.x, fragment.y, object_id);
                    let surface = shade(&ShadeInput {
                        fragment,
                        material: &material,
                        light: &light,
                        time,
                        normal_map: normal_map.filter(|_| normal_mapping_enabled),
                    });
                    let realistic_normal = match material.shader {
                        ShaderKind::RockyPlanet | ShaderKind::GasPlanet => {
                            Some((fragment.world_position - center).normalize())
                        }
                        ShaderKind::CorePlanet | ShaderKind::DroidPlanet => Some(fragment.normal),
                        _ => None,
                    };
                    let color = if let Some(normal) = realistic_normal {
                        let realistic = realistic_color(
                            surface.color,
                            surface.intensity,
                            normal,
                            fragment.world_position,
                            &light,
                            eye,
                        );
                        if dithering_enabled {
                            dither_levels(realistic, REALISTIC_DITHER_LEVELS, fragment.x, fragment.y)
                        } else {
                            Color::from_unit(realistic)
                        }
                    } else if material.shader == ShaderKind::GasPlanet {
                        Color::from_unit(surface.color * (0.72 + surface.intensity * 0.28))
                    } else if dithering_enabled {
                        dither(surface, fragment.x, fragment.y)
                    } else {
                        Color::from_unit(surface.color * surface.intensity)
                    };
                    if dithering_enabled || opacity >= 1.0 {
                        Some(FragmentOutput::opaque(color))
                    } else {
                        Some(FragmentOutput { color, opacity })
                    }
                },
            );
            processed_triangles += stats.processed;
            drawn_triangles += stats.drawn;
        }
        self.metrics
            .record_frame(processed_triangles, drawn_triangles, culled_objects);

        if self.heat_haze_enabled {
            self.heat_haze
                .apply(&mut self.framebuffer, &self.object_buffer, objects, time);
        }

        if self.outline_enabled {
            self.draw_outlines(objects, camera);
        }

        for line in self.queued_lines.drain(..) {
            lines::draw_line_3d(
                &mut self.framebuffer,
                &mut self.depth_buffer,
                view_projection,
                line.start,
                line.end,
                line.color,
                line.dashed,
            );
        }

        self.black_hole
            .apply(&mut self.framebuffer, view_projection, time);

        self.warp
            .apply(&mut self.framebuffer, self.skybox.visible_stars());
    }

    fn draw_outlines(&mut self, objects: &[RenderObject], camera: &Camera) {
        find_outline_pixels(
            &self.object_buffer,
            &self.depth_buffer,
            camera.near,
            camera.far,
            &mut self.outline_pixels,
        );
        for pixel in &self.outline_pixels {
            let material = objects[pixel.object_id as usize - 1].material;
            if material.outline {
                let color = Color::from_unit(material.base_color + material.emissive);
                self.framebuffer.set_pixel(pixel.x, pixel.y, color);
            }
        }
    }

    pub fn draw_line_3d(&mut self, start: Vec3, end: Vec3, color: Color, dashed: bool) {
        self.queued_lines.push(QueuedLine { start, end, color, dashed });
    }

    pub fn draw_line_2d(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        lines::draw_line_2d(&mut self.framebuffer, x0, y0, x1, y1, color);
    }

    pub fn draw_rect_2d(&mut self, x: i32, y: i32, width: i32, height: i32, color: Color) {
        for row in y..y + height {
            for column in x..x + width {
                self.framebuffer.set_pixel(column, row, color);
            }
        }
    }

    pub fn draw_metrics(&mut self, color: Color) {
        self.metrics.draw(&mut self.framebuffer, color);
    }

    pub fn draw_crosshair(&mut self, color: Color) {
        let center_x = self.framebuffer.width() as i32 / 2;
        let center_y = self.framebuffer.height() as i32 / 2;
        let radius = CROSSHAIR_RADIUS;
        self.draw_line_2d(center_x - radius, center_y, center_x + radius, center_y, color);
        self.draw_line_2d(center_x, center_y - radius, center_x, center_y + radius, color);
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

const ICOSAHEDRON_FACES: [[u32; 3]; 20] = [
    [0, 11, 5],
    [0, 5, 1],
    [0, 1, 7],
    [0, 7, 10],
    [0, 10, 11],
    [1, 5, 9],
    [5, 11, 4],
    [11, 10, 2],
    [10, 7, 6],
    [7, 1, 8],
    [3, 9, 4],
    [3, 4, 2],
    [3, 2, 6],
    [3, 6, 8],
    [3, 8, 9],
    [4, 9, 5],
    [2, 4, 11],
    [6, 2, 10],
    [8, 6, 7],
    [9, 8, 1],
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deformation {
    pub seed: u32,
    pub amplitude: f32,
    pub frequency: f32,
    pub crater_count: u32,
    pub crater_radius: f32,
    pub crater_depth: f32,
}

impl Deformation {
    fn crater_centers(&self) -> Vec<Vec3> {
        let mut random = SeededRandom { state: self.seed };
        (0..self.crater_count)
            .map(|_| {
                let height = random.next() * 2.0 - 1.0;
                let azimuth = random.next() * TAU;
                let ring = (1.0 - height * height).sqrt();
                Vec3::new(ring * azimuth.cos(), height, ring * azimuth.sin())
            })
            .collect()
    }

    fn crater_height(&self, direction: Vec3, center: Vec3) -> f32 {
        let distance = (direction - center).length() / self.crater_radius;
        if distance < 1.0 {
            -self.crater_depth * (1.0 - distance * distance)
        } else if distance < CRATER_RIM_END {
            let fade = (CRATER_RIM_END - distance) / (CRATER_RIM_END - 1.0);
            self.crater_depth * CRATER_RIM_HEIGHT * fade
        } else {
            0.0
        }
    }

    fn height(&self, direction: Vec3, craters: &[Vec3]) -> f32 {
        let offset = Vec3::new(self.seed as f32 * 0.137, self.seed as f32 * 0.071, 0.0);
        let terrain = (fbm(direction * self.frequency + offset) - 0.5) * 2.0 * self.amplitude;
        let carved: f32 = craters
            .iter()
            .map(|&center| self.crater_height(direction, center))
            .sum();
        terrain + carved
    }
}

fn sphere_vertex(direction: Vec3) -> Vertex {
    let position = direction.normalize();
    Vertex {
        position,
        normal: position,
        uv: Vec2::new(
            0.5 + position.z.atan2(position.x) / TAU,
            0.5 - position.y.asin() / PI,
        ),
    }
}

impl Mesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn recalculate_normals(&mut self) {
        for vertex in &mut self.vertices {
            vertex.normal = Vec3::default();
        }
        for triangle in self.indices.chunks_exact(3) {
            let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(|index| index as usize);
            let pa = self.vertices[a].position;
            let face = (self.vertices[b].position - pa).cross(self.vertices[c].position - pa);
            for index in [a, b, c] {
                self.vertices[index].normal = self.vertices[index].normal + face;
            }
        }
        for vertex in &mut self.vertices {
            vertex.normal = vertex.normal.normalize();
        }
    }

    pub fn deform(&mut self, deformation: &Deformation) {
        let craters = deformation.crater_centers();
        for vertex in &mut self.vertices {
            let direction = vertex.position.normalize();
            vertex.position = direction * (1.0 + deformation.height(direction, &craters));
        }
        self.recalculate_normals();
    }

    pub fn icosphere(subdivisions: u32) -> Self {
        let t = (1.0 + 5.0_f32.sqrt()) * 0.5;
        let corners = [
            (-1.0, t, 0.0),
            (1.0, t, 0.0),
            (-1.0, -t, 0.0),
            (1.0, -t, 0.0),
            (0.0, -1.0, t),
            (0.0, 1.0, t),
            (0.0, -1.0, -t),
            (0.0, 1.0, -t),
            (t, 0.0, -1.0),
            (t, 0.0, 1.0),
            (-t, 0.0, -1.0),
            (-t, 0.0, 1.0),
        ];
        let mut mesh = Self {
            vertices: corners
                .iter()
                .map(|&(x, y, z)| sphere_vertex(Vec3::new(x, y, z)))
                .collect(),
            indices: ICOSAHEDRON_FACES.concat(),
        };
        for _ in 0..subdivisions {
            mesh.subdivide_sphere();
        }
        mesh
    }

    fn subdivide_sphere(&mut self) {
        let mut midpoints: HashMap<(u32, u32), u32> = HashMap::new();
        let triangles = std::mem::take(&mut self.indices);
        for triangle in triangles.chunks_exact(3) {
            let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
            let ab = self.edge_midpoint(&mut midpoints, a, b);
            let bc = self.edge_midpoint(&mut midpoints, b, c);
            let ca = self.edge_midpoint(&mut midpoints, c, a);
            self.indices
                .extend([a, ab, ca, b, bc, ab, c, ca, bc, ab, bc, ca]);
        }
    }

    fn edge_midpoint(&mut self, midpoints: &mut HashMap<(u32, u32), u32>, a: u32, b: u32) -> u32 {
        let key = (a.min(b), a.max(b));
        if let Some(&index) = midpoints.get(&key) {
            return index;
        }
        let middle = self.vertices[a as usize].position + self.vertices[b as usize].position;
        let index = self.vertices.len() as u32;
        self.vertices.push(sphere_vertex(middle));
        midpoints.insert(key, index);
        index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deformation_is_reproducible_and_bends_normals() {
        let deformation = Deformation {
            seed: 3,
            amplitude: 0.2,
            frequency: 3.0,
            crater_count: 4,
            crater_radius: 0.4,
            crater_depth: 0.1,
        };
        let mut first = Mesh::icosphere(2);
        let mut second = Mesh::icosphere(2);
        first.deform(&deformation);
        second.deform(&deformation);
        assert_eq!(first, second);
        assert!(first
            .vertices
            .iter()
            .any(|vertex| vertex.normal.dot(vertex.position.normalize()) < 0.99));
    }

    #[test]
    fn subdivision_quadruples_triangles_and_keeps_outward_winding() {
        let mesh = Mesh::icosphere(2);
        assert_eq!(mesh.triangle_count(), 320);
        assert_eq!(mesh.vertices.len(), 162);
        for triangle in mesh.indices.chunks_exact(3) {
            let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
                .map(|index| mesh.vertices[index as usize].position);
            assert!((b - a).cross(c - a).dot(a) > 0.0);
        }
    }
}
