use crate::assets::AssetManager;
use crate::engine::camera::Camera;
use crate::engine::math::Vec3;
use crate::engine::scene::Scene;
use crate::game::hud::draw_hud;
use crate::game::{Game, ProgramMode};
use crate::platform::audio::Audio;
use crate::platform::music::MusicTrack;
use crate::platform::input::{InputState, Key};
use crate::platform::time::{Clock, FixedTimestep};
use crate::platform::window::Window;
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use crate::renderer::lighting::PointLight;
use crate::renderer::opening_crawl::OpeningCrawl;
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
    opening: Option<OpeningCrawl>,
    audio: Option<Audio>,
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
            opening: Some(OpeningCrawl::new()),
            audio: Audio::open(),
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
            if let Some(audio) = &mut self.audio {
                audio.set_music(Some(if self.opening.is_some() {
                    MusicTrack::Intro
                } else {
                    match self.game.program_mode {
                        ProgramMode::Menu => MusicTrack::Menu,
                        ProgramMode::Game => MusicTrack::Battle,
                    }
                }));
                audio.update(frame_delta);
                for effect in self.game.take_sounds() {
                    audio.play(effect);
                }
            }
            self.renderer.metrics.update(frame_delta);
            self.draw();
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.simulation_time += delta_time;
        if let Some(opening) = &mut self.opening {
            let skip = [Key::Enter, Key::Escape, Key::Space]
                .iter()
                .any(|&key| self.input.was_pressed(key));
            opening.update(delta_time, skip);
            self.game.system.update(delta_time);
            if opening.is_finished() {
                self.opening = None;
                self.renderer.warp.start_warp();
            }
            return;
        }
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
        let (black_hole_center, black_hole_intensity) = self.game.black_hole();
        self.renderer.black_hole.center = black_hole_center;
        self.renderer.black_hole.intensity = if self.opening.is_some() {
            0.0
        } else {
            black_hole_intensity
        };
        let camera = match &self.opening {
            Some(opening) => opening.camera(&self.camera),
            None => self.camera,
        };
        self.renderer
            .render(&objects, &camera, self.simulation_time);
        if let Some(opening) = &self.opening {
            opening.draw(self.renderer.framebuffer_mut());
            self.window.present(self.renderer.pixels(), &[]);
            return;
        }
        if self.game.show_metrics {
            self.renderer.draw_metrics(METRICS_COLOR);
        }
        let (damage_color, damage_amount) = self.game.damage_flash();
        self.renderer.tint_screen(damage_color, damage_amount);
        let flash = self.game.transition_flash();
        let labels = if flash > 0.0 {
            Vec::new()
        } else {
            let mut labels = draw_hud(&mut self.renderer, &self.game.hud_info());
            labels.extend(self.game.damage_labels(&self.camera));
            labels.extend(self.game.draw_overlays(&mut self.renderer));
            labels
        };
        self.renderer.fade_to_white(flash);
        self.window.present(self.renderer.pixels(), &labels);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
