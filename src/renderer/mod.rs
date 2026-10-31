pub mod clipping;
pub mod culling;
pub mod depth;
pub mod dithering;
pub mod framebuffer;
pub mod lighting;
pub mod lines;
pub mod outline;
pub mod pipeline;
pub mod rasterizer;
pub mod shaders;
pub mod skybox;
pub mod warp_effect;

use crate::engine::camera::Camera;
use crate::engine::math::{Mat4, Vec2, Vec3};
use depth::DepthBuffer;
use dithering::dither;
use framebuffer::{Color, Framebuffer};
use lighting::PointLight;
use outline::{find_outline_pixels, ObjectBuffer, OutlinePixel};
use shaders::{shade, ShadeInput, ShaderKind};
use skybox::Skybox;
use warp_effect::WarpEffect;

const DEFAULT_AMBIENT: f32 = 0.12;
const CROSSHAIR_RADIUS: i32 = 4;
const STAR_SEED: u32 = 20_260_001;
const STAR_COUNT: usize = 1800;

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
    pub outline_enabled: bool,
    pub warp: WarpEffect,
    framebuffer: Framebuffer,
    depth_buffer: DepthBuffer,
    object_buffer: ObjectBuffer,
    outline_pixels: Vec<OutlinePixel>,
    skybox: Skybox,
    queued_lines: Vec<QueuedLine>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            light: PointLight::new(Vec3::default(), DEFAULT_AMBIENT),
            dithering_enabled: true,
            outline_enabled: true,
            warp: WarpEffect::new(),
            framebuffer: Framebuffer::new(),
            depth_buffer: DepthBuffer::new(),
            object_buffer: ObjectBuffer::new(),
            outline_pixels: Vec::new(),
            skybox: Skybox::new(STAR_SEED, STAR_COUNT),
            queued_lines: Vec::new(),
        }
    }

    pub fn pixels(&self) -> &[u8] {
        self.framebuffer.pixels()
    }

    pub fn render(&mut self, objects: &[RenderObject], camera: &Camera, time: f32) {
        self.framebuffer.clear(Color::BLACK);
        self.depth_buffer.clear();
        self.skybox.draw(&mut self.framebuffer, camera, time);
        let view_projection = camera.view_projection_matrix();
        let light = self.light;
        let dithering_enabled = self.dithering_enabled;

        self.object_buffer.clear();
        let object_buffer = &mut self.object_buffer;

        for (index, object) in objects.iter().enumerate() {
            let material = object.material;
            let object_id = index as u32 + 1;
            pipeline::draw_mesh(
                &mut self.framebuffer,
                &mut self.depth_buffer,
                object.mesh,
                object.model_matrix,
                view_projection,
                |fragment| {
                    object_buffer.set(fragment.x, fragment.y, object_id);
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
