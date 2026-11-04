use crate::assets::material::{glowing, ShaderKind};
use crate::assets::{AssetManager, MeshHandle};
use crate::engine::camera::Camera;
use crate::engine::math::{Mat4, Vec3};
use crate::platform::input::{InputState, Key};
use crate::platform::time::{Clock, FixedTimestep};
use crate::platform::window::Window;
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use crate::renderer::{Material, RenderObject, Renderer};

pub const SIMULATION_STEP: f32 = 1.0 / 60.0;
pub const MAX_STEPS_PER_FRAME: u32 = 5;
const LOOK_SPEED: f32 = 1.2;
const CROSSHAIR_COLOR: Color = Color::new(255, 255, 255);
const SHIP_COLOR: Vec3 = Vec3::new(0.36, 1.0, 0.36);
const SHIP_GLOW: Vec3 = Vec3::new(0.08, 0.25, 0.08);
const SHIP_RADIUS: f32 = 1.5;
const SHIP_DISTANCE_AHEAD: f32 = 7.0;
const SHIP_DISTANCE_BELOW: f32 = 1.8;

pub struct App {
    window: Window,
    renderer: Renderer,
    camera: Camera,
    input: InputState,
    assets: AssetManager,
    ship_mesh: MeshHandle,
    ship_material: Material,
    clock: Clock,
    timestep: FixedTimestep,
    simulation_time: f32,
}

impl App {
    pub fn new() -> Self {
        let aspect = FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32;
        let mut assets = AssetManager::new();
        let ship_mesh = assets.load_ship();
        Self {
            window: Window::open(),
            renderer: Renderer::new(),
            camera: Camera::new(Vec3::default(), aspect),
            input: InputState::new(),
            assets,
            ship_mesh,
            ship_material: glowing(SHIP_COLOR, SHIP_GLOW, ShaderKind::Lambert),
            clock: Clock::start(),
            timestep: FixedTimestep::new(SIMULATION_STEP, MAX_STEPS_PER_FRAME),
            simulation_time: 0.0,
        }
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {
            let frame_delta = self.clock.tick();
            self.input.capture(self.window.handle());
            let steps = self.timestep.advance(frame_delta);
            for _ in 0..steps {
                self.update(self.timestep.step());
                self.input.consume_events();
            }
            self.renderer.metrics.update(frame_delta);
            self.draw();
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.simulation_time += delta_time;
        let yaw = self.input.axis(Key::Left, Key::Right);
        let pitch = self.input.axis(Key::Up, Key::Down);
        self.camera.rotate(
            yaw * LOOK_SPEED * delta_time,
            pitch * LOOK_SPEED * delta_time,
            0.0,
        );
        self.renderer.warp.update(delta_time);
    }

    fn ship_model_matrix(&self) -> Mat4 {
        let forward = self.camera.forward();
        let up = self.camera.up();
        let right = self.camera.right();
        let back = -forward;
        let position =
            self.camera.position + forward * SHIP_DISTANCE_AHEAD - up * SHIP_DISTANCE_BELOW;
        let radius = self.assets.mesh_bounding_radius(self.ship_mesh);
        let scale = SHIP_RADIUS / radius.max(f32::EPSILON);
        Mat4 {
            rows: [
                [right.x, up.x, back.x, position.x],
                [right.y, up.y, back.y, position.y],
                [right.z, up.z, back.z, position.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        } * Mat4::scale(Vec3::new(scale, scale, scale))
    }

    fn draw(&mut self) {
        let ship = RenderObject {
            mesh: self.assets.mesh(self.ship_mesh),
            model_matrix: self.ship_model_matrix(),
            material: self.ship_material,
            bounding_radius: SHIP_RADIUS,
            lod: None,
        };
        self.renderer
            .render(&[ship], &self.camera, self.simulation_time);
        self.renderer.draw_crosshair(CROSSHAIR_COLOR);
        self.window.present(self.renderer.pixels());
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
