pub mod clipping;
pub mod culling;
pub mod depth;
pub mod dithering;
pub mod framebuffer;
pub mod lighting;
pub mod lines;
pub mod pipeline;
pub mod rasterizer;
pub mod shaders;

use crate::engine::camera::Camera;
use crate::engine::math::{Mat4, Vec2, Vec3};
use depth::DepthBuffer;
use dithering::dither;
use framebuffer::{Color, Framebuffer};
use lighting::PointLight;
use shaders::{shade, ShadeInput, ShaderKind};

const DEFAULT_AMBIENT: f32 = 0.12;
const CROSSHAIR_RADIUS: i32 = 4;

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
}

#[derive(Debug, Clone, Copy)]
pub struct RenderObject<'a> {
    pub mesh: &'a Mesh,
    pub model_matrix: Mat4,
    pub material: Material,
    pub bounding_radius: f32,
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
    framebuffer: Framebuffer,
    depth_buffer: DepthBuffer,
    queued_lines: Vec<QueuedLine>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            light: PointLight::new(Vec3::default(), DEFAULT_AMBIENT),
            dithering_enabled: true,
            framebuffer: Framebuffer::new(),
            depth_buffer: DepthBuffer::new(),
            queued_lines: Vec::new(),
        }
    }

    pub fn pixels(&self) -> &[u8] {
        self.framebuffer.pixels()
    }

    pub fn render(&mut self, objects: &[RenderObject], camera: &Camera, time: f32) {
        self.framebuffer.clear(Color::BLACK);
        self.depth_buffer.clear();
        let view_projection = camera.view_projection_matrix();
        let light = self.light;
        let dithering_enabled = self.dithering_enabled;

        for object in objects {
            let material = object.material;
            pipeline::draw_mesh(
                &mut self.framebuffer,
                &mut self.depth_buffer,
                object.mesh,
                object.model_matrix,
                view_projection,
                |fragment| {
                    let surface = shade(&ShadeInput {
                        fragment,
                        material: &material,
                        light: &light,
                        time,
                    });
                    if dithering_enabled {
                        dither(surface, fragment.x, fragment.y)
                    } else {
                        Color::from_unit(surface.color * surface.intensity)
                    }
                },
            );
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
