use crate::assets::AssetManager;
use crate::engine::camera::Camera;
use crate::engine::math::Vec3;
use crate::engine::scene::Scene;
use crate::game::hud::draw_hud;
use crate::game::Game;
use crate::platform::input::InputState;
use crate::platform::time::{Clock, FixedTimestep};
use crate::platform::window::Window;
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use crate::renderer::lighting::PointLight;
use crate::renderer::Renderer;

pub const SIMULATION_STEP: f32 = 1.0 / 60.0;
pub const MAX_STEPS_PER_FRAME: u32 = 5;
const METRICS_COLOR: Color = Color::new(92, 255, 92);

pub struct App {
    window: Window,
    renderer: Renderer,
    camera: Camera,
    input: InputState,
    assets: AssetManager,
    scene: Scene,
    game: Game,
    clock: Clock,
    timestep: FixedTimestep,
    simulation_time: f32,
}

impl App {
    pub fn new() -> Self {
        let window = Window::open();
        let aspect = FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32;
        let mut camera = Camera::new(Vec3::default(), aspect);
        let mut assets = AssetManager::new();
        let game = Game::new(&mut assets, &mut camera);
        let mut renderer = Renderer::new();
        renderer.heat_haze_enabled = true;
        Self {
            window,
            renderer,
            camera,
            input: InputState::new(),
            assets,
            scene: Scene::new(),
            game,
            clock: Clock::start(),
            timestep: FixedTimestep::new(SIMULATION_STEP, MAX_STEPS_PER_FRAME),
            simulation_time: 0.0,
        }
    }

    pub fn run(&mut self) {
        while !self.window.should_close() && !self.game.quit_requested() {
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
        self.game.update(
            &self.input,
            delta_time,
            &mut self.camera,
            &mut self.renderer.warp,
        );
    }

    fn draw(&mut self) {
        self.renderer.light = PointLight::new(
            self.game.light_position(),
            self.game.system.config.light_ambient,
        );
        self.game.populate(&mut self.scene, &self.assets);
        self.game.queue_lines(&mut self.renderer);
        let objects = self.scene.render_objects(&self.assets);
        self.renderer
            .render(&objects, &self.camera, self.simulation_time);
        if self.game.show_metrics {
            self.renderer.draw_metrics(METRICS_COLOR);
        }
        let labels = draw_hud(&mut self.renderer, &self.game.hud_info());
        self.window.present(self.renderer.pixels(), &labels);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
