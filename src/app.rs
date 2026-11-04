use crate::engine::camera::Camera;
use crate::engine::math::Vec3;
use crate::platform::input::{InputState, Key};
use crate::platform::time::{Clock, FixedTimestep};
use crate::platform::window::Window;
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use crate::renderer::Renderer;

pub const SIMULATION_STEP: f32 = 1.0 / 60.0;
pub const MAX_STEPS_PER_FRAME: u32 = 5;
const LOOK_SPEED: f32 = 1.2;
const CROSSHAIR_COLOR: Color = Color::new(255, 255, 255);

pub struct App {
    window: Window,
    renderer: Renderer,
    camera: Camera,
    input: InputState,
    clock: Clock,
    timestep: FixedTimestep,
    simulation_time: f32,
}

impl App {
    pub fn new() -> Self {
        let aspect = FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32;
        Self {
            window: Window::open(),
            renderer: Renderer::new(),
            camera: Camera::new(Vec3::default(), aspect),
            input: InputState::new(),
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

    fn draw(&mut self) {
        self.renderer
            .render(&[], &self.camera, self.simulation_time);
        self.renderer.draw_crosshair(CROSSHAIR_COLOR);
        self.window.present(self.renderer.pixels());
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
