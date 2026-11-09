pub mod combat;
pub mod enemies;
pub mod hud;
pub mod player;
pub mod progression;
pub mod session;
pub mod sun_lifecycle;
pub mod warp;

use crate::assets::material::{glowing, solid};
use crate::assets::mesh::{crystal, placeholder_ship, turned_around};
use crate::assets::{AssetManager, MeshHandle};
use crate::engine::camera::Camera;
use crate::engine::collision::{resolve_all, Collider};
use crate::engine::math::{Mat4, Vec3};
use crate::engine::scene::{Entity, Model, Scene};
use crate::engine::transform::{basis_rotation, rotate_vector, Transform};
use crate::platform::input::{InputState, Key, MouseButton};
use crate::renderer::framebuffer::Color;
use crate::renderer::shaders::ShaderKind;
use crate::renderer::warp_effect::WarpEffect;
use crate::renderer::{Deformation, Material, Mesh, Renderer};
use crate::world::generation::{asteroid_belt, scatter_surface, Asteroid, PropKind, SurfaceProp};
use crate::world::orbit::Orbit;
use crate::world::solar_system::MAX_TIME_SCALE;
use crate::world::{SolarSystem, SystemConfig};
use combat::WeaponSystem;
use enemies::{EnemySwarm, ENEMY_RENDER_SCALE};
use hud::HudInfo;
use player::{FlightMode, Ship, ShipControls, SHIP_RADIUS};
use progression::PlayerProgression;
use session::{generate_seed, RunSession, RunState};
use std::f32::consts::PI;
use sun_lifecycle::{follow_offset, radius_multiplier};
use warp::{Destination, Warp, WarpEvent, WarpState};

const SPHERE_SUBDIVISIONS: [u32; 3] = [4, 3, 1];
const ASTEROID_SUBDIVISIONS: [u32; 3] = [2, 1, 0];
const ENEMY_MODEL_FILE: &str = "ENEMY1.obj";
const SURFACE_CRYSTAL_FILE: &str = "crystal.obj";
const ORBIT_CAMERA_DISTANCE_FACTOR: f32 = 1.25;
const ORBIT_CAMERA_HEIGHT_RATIO: f32 = 0.22;
const ORBIT_CAMERA_SMOOTHING: f32 = 3.2;
const MODEL_FORWARD_CORRECTION: f32 = PI;
const ENEMY_SCALE: f32 = 0.8;
const COLLISION_MARGIN: f32 = 0.4;
const CAMERA_RADIUS: f32 = 0.3;
const CAMERA_FAR: f32 = 1_200.0;
const MENU_CAMERA_FAR: f32 = 900.0;
const FOLLOW_HEIGHT_RATIO: f32 = 0.36;
const FOLLOW_LOOK_AHEAD: f32 = 9.0;
const FOLLOW_SMOOTHING: f32 = 6.0;
const DEFAULT_FOLLOW_DISTANCE: f32 = 4.6;
const MIN_FOLLOW_DISTANCE: f32 = 3.0;
const MAX_FOLLOW_DISTANCE: f32 = 14.0;
const WHEEL_ZOOM_STEP: f32 = 0.8;
const MOUSE_SENSITIVITY: f32 = 0.10;
const FREE_CAMERA_SPEED: f32 = 12.0;
const FREE_CAMERA_BOOST: f32 = 36.0;
const FREE_CAMERA_YAW_RATE: f32 = 1.4;
const FREE_CAMERA_PITCH_RATE: f32 = 1.1;
const FREE_CAMERA_ROLL_RATE: f32 = 1.6;
const TIME_SCALE_STEP: f32 = 0.25;
const NOTICE_DURATION: f32 = 3.5;
const WELCOME_DURATION: f32 = 7.0;
const ORBIT_SEGMENTS: usize = 128;
const ORBIT_COLOR: Color = Color::new(192, 80, 255);
const DESTINATION_COLOR: Color = Color::new(92, 255, 92);
const START_POSITION: Vec3 = Vec3::new(0.0, 10.0, 104.0);
const MENU_START_POSITION: Vec3 = Vec3::new(0.0, 20.0, 220.0);
const START_DESTINATION: usize = 3;
const SUN_ENTRY_DEPTH: f32 = 0.78;
const START_FLASH_IN_DURATION: f32 = 0.55;
const START_FLASH_HOLD_DURATION: f32 = 0.35;
const START_FLASH_OUT_DURATION: f32 = 0.70;
const START_FLASH_DURATION: f32 =
    START_FLASH_IN_DURATION + START_FLASH_HOLD_DURATION + START_FLASH_OUT_DURATION;
const PLAYER_COLOR: Vec3 = Vec3::new(0.36, 1.0, 0.36);
const PLAYER_GLOW: Vec3 = Vec3::new(0.05, 0.2, 0.05);
const ENEMY_COLOR: Vec3 = Vec3::new(0.46, 0.11, 0.68);
const SAND_COLOR: Vec3 = Vec3::new(0.85, 0.76, 0.6);
const CRYSTAL_COLOR: Vec3 = Vec3::new(0.75, 0.31, 1.0);
const CRYSTAL_GLOW: Vec3 = Vec3::new(0.12, 0.03, 0.18);
const BOULDER_COLOR: Vec3 = Vec3::new(0.62, 0.5, 0.38);
const PROJECTILE_COLOR: Color = Color::new(255, 32, 32);

const ASTEROID_DEFORMATION: Deformation = Deformation {
    seed: 11,
    amplitude: 0.35,
    frequency: 2.5,
    crater_count: 4,
    crater_radius: 0.45,
    crater_depth: 0.12,
};

const BOULDER_DEFORMATION: Deformation = Deformation {
    seed: 29,
    amplitude: 0.3,
    frequency: 3.0,
    crater_count: 0,
    crater_radius: 0.3,
    crater_depth: 0.0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Follow,
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramMode {
    Menu,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StartTransition {
    Idle,
    Flashing { elapsed: f32, switched: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatrolEnemy {
    pub host: usize,
    pub orbit: Orbit,
    pub scale: f32,
}

struct Models {
    bodies: Vec<Model>,
    ship: MeshHandle,
    enemy: MeshHandle,
    asteroid: Model,
    crystal: Model,
    boulder: Model,
}

struct Materials {
    ship: Material,
    enemy: Material,
    asteroid: Material,
    crystal: Material,
    boulder: Material,
}

struct Surface {
    body: usize,
    props: Vec<SurfaceProp>,
    embed_depth: f32,
}

pub struct Game {
    pub system: SolarSystem,
    pub ship: Ship,
    pub warp: Warp,
    pub camera_mode: CameraMode,
    pub program_mode: ProgramMode,
    pub destination: usize,
    pub show_help: bool,
    pub show_metrics: bool,
    pub session: RunSession,
    pub progression: PlayerProgression,
    next_run_seed: u32,
    seed_input: Option<String>,
    sun_base_radius: f32,
    asteroids: Vec<Asteroid>,
    surfaces: Vec<Surface>,
    menu_patrols: Vec<PatrolEnemy>,
    run_enemies: EnemySwarm,
    weapons: WeaponSystem,
    models: Models,
    materials: Materials,
    colliders: Vec<Collider>,
    follow_distance: f32,
    notice: Option<(String, f32)>,
    quit_requested: bool,
    time: f32,
    start_transition: StartTransition,
    menu_orbit_lockout: Option<usize>,
}

fn sphere_lod(
    subdivisions: [u32; 3],
    deformation: Option<Deformation>,
) -> impl FnMut(usize) -> Mesh {
    move |level| {
        let mut mesh = Mesh::icosphere(subdivisions[level]);
        if let Some(deformation) = deformation {
            mesh.deform(&deformation);
        }
        mesh
    }
}

fn load_models(assets: &mut AssetManager, system: &SolarSystem) -> Models {
    let sphere =
        Model::Lod(assets.lod_or_insert_with("esfera", sphere_lod(SPHERE_SUBDIVISIONS, None)));
    let bodies = (0..system.body_count())
        .map(|index| match system.body(index).model.file_name() {
            Some(file_name) => Model::Mesh(
                assets.load_model_or_else(file_name, || Mesh::icosphere(SPHERE_SUBDIVISIONS[0])),
            ),
            None => sphere,
        })
        .collect();
    Models {
        bodies,
        ship: assets.load_ship(),
        enemy: assets.load_model_or_else(ENEMY_MODEL_FILE, || turned_around(placeholder_ship())),
        asteroid: Model::Lod(assets.lod_or_insert_with(
            "asteroide",
            sphere_lod(ASTEROID_SUBDIVISIONS, Some(ASTEROID_DEFORMATION)),
        )),
        crystal: Model::Mesh(assets.optional_model_or_else(SURFACE_CRYSTAL_FILE, crystal)),
        boulder: Model::Lod(assets.lod_or_insert_with(
            "roca_superficie",
            sphere_lod(ASTEROID_SUBDIVISIONS, Some(BOULDER_DEFORMATION)),
        )),
    }
}

fn patrols(system: &SolarSystem) -> Vec<PatrolEnemy> {
    let plan: [(&str, f32, f32, f32, f32); 6] = [
        ("ROCOSO", 2.4, 0.42, 0.0, 0.5),
        ("ROCOSO", 3.1, -0.3, 2.5, -0.9),
        ("GASEOSO", 2.2, 0.28, 1.0, 0.3),
        ("GASEOSO", 2.8, -0.22, 4.0, -0.6),
        ("DROIDE", 1.9, 0.25, 0.5, 0.15),
        ("NUCLEO", 2.6, 0.35, 3.3, 1.1),
    ];
    plan.iter()
        .filter_map(|&(name, distance, speed, phase, tilt)| {
            let host = system.find(name)?;
            let radius = system.body(host).radius * distance;
            Some(PatrolEnemy {
                host,
                orbit: Orbit::circular(radius, speed, phase).with_tilt(tilt, phase),
                scale: ENEMY_SCALE,
            })
        })
        .collect()
}

fn surfaces(system: &SolarSystem) -> Vec<Surface> {
    (0..system.body_count())
        .filter_map(|body| {
            let config = system.body(body).surface?;
            Some(Surface {
                body,
                props: scatter_surface(&config),
                embed_depth: config.embed_depth,
            })
        })
        .collect()
}

fn apply_run_seed(config: &mut SystemConfig, seed: u32) {
    config.belt.seed = mix_seed(seed, 0);
    for (index, body) in config.bodies.iter_mut().enumerate() {
        if let Some(surface) = &mut body.surface {
            // Surface noise converts this value to spatial coordinates, so keep
            // it bounded to avoid overflowing its integer lattice.
            surface.seed = mix_seed(seed, index as u32 + 1) % 1_000_000;
        }
    }
}

fn mix_seed(seed: u32, stream: u32) -> u32 {
    let mut value = seed ^ stream.wrapping_mul(0x9E37_79B9);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^ (value >> 16)
}

impl Game {
    pub fn new(assets: &mut AssetManager, camera: &mut Camera) -> Self {
        let system = SolarSystem::new(SystemConfig::menu());
        let models = load_models(assets, &system);
        let asteroids = asteroid_belt(&system.config.belt);
        let surfaces = surfaces(&system);
        let menu_patrols = patrols(&system);
        let destination = START_DESTINATION.min(system.body_count() - 1);
        let sun_base_radius = system.body(0).radius;
        let ship = Ship::new(
            MENU_START_POSITION,
            (system.sun_position() - MENU_START_POSITION).normalize(),
        );
        camera.far = MENU_CAMERA_FAR;
        camera.position = ship.position - ship.forward() * DEFAULT_FOLLOW_DISTANCE
            + ship.up() * (DEFAULT_FOLLOW_DISTANCE * FOLLOW_HEIGHT_RATIO);
        camera.look_at(ship.position + ship.forward() * FOLLOW_LOOK_AHEAD);
        let mut game = Self {
            system,
            ship,
            warp: Warp::new(),
            camera_mode: CameraMode::Follow,
            program_mode: ProgramMode::Menu,
            destination,
            show_help: false,
            show_metrics: false,
            session: RunSession::new(),
            progression: PlayerProgression::new(),
            next_run_seed: generate_seed(),
            seed_input: None,
            sun_base_radius,
            asteroids,
            surfaces,
            menu_patrols,
            run_enemies: EnemySwarm::new(),
            weapons: WeaponSystem::new(),
            models,
            materials: Materials {
                ship: glowing(PLAYER_COLOR, PLAYER_GLOW, ShaderKind::EtaPlayer),
                enemy: solid(ENEMY_COLOR, ShaderKind::EnemyShip),
                asteroid: solid(SAND_COLOR, ShaderKind::Rocky),
                crystal: glowing(CRYSTAL_COLOR, CRYSTAL_GLOW, ShaderKind::Lambert),
                boulder: solid(BOULDER_COLOR, ShaderKind::Rocky),
            },
            colliders: Vec::new(),
            follow_distance: DEFAULT_FOLLOW_DISTANCE,
            notice: None,
            quit_requested: false,
            time: 0.0,
            start_transition: StartTransition::Idle,
            menu_orbit_lockout: None,
        };
        game.notify_for(
            "MENU: EXPLORA EL SISTEMA Y VUELA DENTRO DEL SOL PARA EMPEZAR",
            WELCOME_DURATION,
        );
        game
    }

    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    pub fn body_name(&self, index: usize) -> &'static str {
        self.system.body(index).name
    }

    fn notify(&mut self, message: impl Into<String>) {
        self.notify_for(message, NOTICE_DURATION);
    }

    fn notify_for(&mut self, message: impl Into<String>, duration: f32) {
        self.notice = Some((message.into(), duration));
    }

    fn configure_mode(&mut self, mode: ProgramMode, camera: &mut Camera) {
        let (mut config, start, camera_far, welcome) = match mode {
            ProgramMode::Menu => (
                SystemConfig::menu(),
                MENU_START_POSITION,
                MENU_CAMERA_FAR,
                "MENU: VUELA DENTRO DEL SOL PARA EMPEZAR",
            ),
            ProgramMode::Game => (
                SystemConfig::game(),
                START_POSITION,
                CAMERA_FAR,
                "JUEGO INICIADO - PULSA H PARA VER LOS CONTROLES",
            ),
        };
        self.program_mode = mode;
        self.progression.reset();
        match mode {
            ProgramMode::Menu => {
                self.session.reset();
                self.run_enemies.clear();
                self.weapons.reset();
            }
            ProgramMode::Game => {
                self.session.start(self.next_run_seed);
                self.run_enemies.reset(self.next_run_seed);
                self.weapons.reset();
                apply_run_seed(&mut config, self.next_run_seed);
            }
        }
        self.seed_input = None;
        self.sun_base_radius = config.bodies[0].radius;
        self.system = SolarSystem::new(config);
        self.asteroids = asteroid_belt(&self.system.config.belt);
        self.surfaces = surfaces(&self.system);
        self.menu_patrols = patrols(&self.system);
        self.ship = Ship::new(start, (self.system.sun_position() - start).normalize());
        self.warp = Warp::new();
        self.destination = START_DESTINATION.min(self.system.body_count() - 1);
        self.camera_mode = CameraMode::Follow;
        self.show_help = false;
        self.follow_distance = DEFAULT_FOLLOW_DISTANCE;
        self.colliders.clear();
        self.menu_orbit_lockout = None;
        camera.far = camera_far;
        camera.position = self.ship.position - self.ship.forward() * DEFAULT_FOLLOW_DISTANCE
            + self.ship.up() * (DEFAULT_FOLLOW_DISTANCE * FOLLOW_HEIGHT_RATIO);
        camera.look_at(self.ship.position + self.ship.forward() * FOLLOW_LOOK_AHEAD);
        self.notify_for(welcome, WELCOME_DURATION);
    }

    pub fn update(
        &mut self,
        input: &InputState,
        delta_time: f32,
        camera: &mut Camera,
        effect: &mut WarpEffect,
    ) {
        self.time += delta_time;
        if !matches!(self.start_transition, StartTransition::Idle) {
            self.update_start_transition(delta_time, camera);
            effect.update(delta_time);
            return;
        }

        self.handle_commands(input, camera, effect);
        if self.program_mode == ProgramMode::Game && self.session.update(delta_time) {
            self.ship.stop();
            self.notify("EL SOL HA COMENZADO A COLAPSAR");
        }
        self.update_sun_lifecycle(delta_time);
        self.system.update(delta_time);
        self.refresh_colliders();

        let run_is_paused =
            self.program_mode == ProgramMode::Game && self.session.state() != RunState::Playing;
        let controls = if self.seed_input.is_some() || run_is_paused {
            ShipControls::default()
        } else {
            ship_controls(input)
        };
        let ship_controls = if self.camera_mode == CameraMode::Free {
            ShipControls::default()
        } else {
            controls
        };

        if self.warp.is_traveling() {
            let target = self.warp.target().unwrap_or(self.destination);
            let destination = self.destination_of(target);
            if let WarpEvent::Arrived(body) =
                self.warp.update(delta_time, &mut self.ship, destination)
            {
                self.notify(format!(
                    "LLEGADA A {} - PULSA O PARA ORBITAR",
                    self.body_name(body)
                ));
            }
        } else {
            let destination = self.destination_of(self.destination);
            self.warp.update(delta_time, &mut self.ship, destination);
            self.fly(&ship_controls, delta_time);
            let first_body = usize::from(self.program_mode == ProgramMode::Menu);
            let obstacles = if self.ship.is_orbiting() {
                &self.colliders[first_body..self.system.body_count()]
            } else {
                &self.colliders[first_body..]
            };
            let (position, velocity, _) = resolve_all(
                self.ship.position,
                self.ship.velocity,
                SHIP_RADIUS,
                obstacles,
                COLLISION_MARGIN,
            );
            self.ship.position = position;
            if !self.ship.is_orbiting() {
                self.ship.velocity = velocity;
            }
            self.update_menu_orbit_lockout();
            self.try_enter_menu_orbit();
            self.try_start_game_transition();
        }

        if self.program_mode == ProgramMode::Game && self.session.state() == RunState::Playing {
            self.weapons.update(
                self.ship.position,
                self.ship.forward(),
                self.ship.right(),
                self.ship.up(),
                &mut self.run_enemies,
                &self.colliders,
                delta_time,
            );
            let enemy_update = self.run_enemies.update(
                self.ship.position,
                self.ship.forward(),
                SHIP_RADIUS,
                self.progression.level(),
                &self.colliders,
                delta_time,
            );
            if enemy_update.contact_damage > 0.0 {
                self.ship.velocity = self.ship.velocity + enemy_update.player_knockback;
                self.damage_player(enemy_update.contact_damage);
            }
        }

        match self.camera_mode {
            CameraMode::Follow => self.follow_ship(camera, delta_time),
            CameraMode::Free => self.fly_free_camera(camera, &controls, delta_time),
        }
        let first_collider = usize::from(self.program_mode == ProgramMode::Menu);
        let (position, _, _) = resolve_all(
            camera.position,
            Vec3::default(),
            CAMERA_RADIUS,
            &self.colliders[first_collider..],
            COLLISION_MARGIN,
        );
        camera.position = position;

        effect.update(delta_time);
        if let Some((_, remaining)) = &mut self.notice {
            *remaining -= delta_time;
            if *remaining <= 0.0 {
                self.notice = None;
            }
        }
    }

    fn refresh_colliders(&mut self) {
        self.colliders = self.system.colliders();
        let time = self.system.orbital_time();
        let origin = self.system.origin();
        self.colliders.extend(
            self.asteroids.iter().map(|asteroid| {
                Collider::sphere(origin + asteroid.position(time), asteroid.radius())
            }),
        );
    }

    pub fn damage_player(&mut self, amount: f32) -> bool {
        if self.program_mode != ProgramMode::Game || self.session.state() != RunState::Playing {
            return false;
        }
        let destroyed = self.progression.take_damage(amount);
        if destroyed {
            self.session.destroy_player();
            self.ship.stop();
            self.notify("NAVE DESTRUIDA");
        }
        destroyed
    }

    pub fn grant_player_experience(&mut self, amount: u32) -> u32 {
        if self.program_mode != ProgramMode::Game || self.session.state() != RunState::Playing {
            return 0;
        }
        let gain = self.progression.grant_experience(amount);
        if gain.levels_gained > 0 {
            self.notify(format!("NIVEL {}", self.progression.level()));
        }
        gain.levels_gained
    }

    fn update_sun_lifecycle(&mut self, delta_time: f32) {
        if self.program_mode != ProgramMode::Game {
            return;
        }
        let offset = follow_offset(self.system.sun_position(), self.ship.position, delta_time);
        self.system.translate_origin(offset);
        self.system.config.bodies[0].radius =
            self.sun_base_radius * radius_multiplier(self.session.elapsed());
    }

    fn destination_of(&self, body: usize) -> Destination {
        Destination {
            center: self.system.position(body),
            radius: self.system.body(body).radius,
        }
    }

    fn handle_commands(
        &mut self,
        input: &InputState,
        camera: &mut Camera,
        effect: &mut WarpEffect,
    ) {
        if self.handle_seed_commands(input) {
            return;
        }
        if input.was_pressed(Key::Escape) {
            if self.show_help {
                self.show_help = false;
            } else if self.program_mode == ProgramMode::Game {
                self.configure_mode(ProgramMode::Menu, camera);
            } else {
                self.quit_requested = true;
            }
        }
        if input.was_pressed(Key::H) {
            self.show_help = !self.show_help;
        }
        if input.was_pressed(Key::M) {
            self.show_metrics = !self.show_metrics;
        }
        if input.was_pressed(Key::C) {
            self.camera_mode = match self.camera_mode {
                CameraMode::Follow => CameraMode::Free,
                CameraMode::Free => CameraMode::Follow,
            };
            let message = match self.camera_mode {
                CameraMode::Follow => "CAMARA SIGUE A LA NAVE",
                CameraMode::Free => "CAMARA LIBRE - LA NAVE QUEDA EN PILOTO AUTOMATICO",
            };
            self.notify(message);
        }
        if input.was_pressed(Key::Minus) || input.was_pressed(Key::Equal) {
            let step = if input.was_pressed(Key::Equal) {
                TIME_SCALE_STEP
            } else {
                -TIME_SCALE_STEP
            };
            self.system.config.time_scale =
                (self.system.config.time_scale + step).clamp(0.0, MAX_TIME_SCALE);
            self.notify(format!(
                "VELOCIDAD DE ORBITAS X{:.2}",
                self.system.config.time_scale
            ));
        }
        let wheel = input.wheel();
        if wheel != 0.0 {
            self.follow_distance = (self.follow_distance - wheel * WHEEL_ZOOM_STEP)
                .clamp(MIN_FOLLOW_DISTANCE, MAX_FOLLOW_DISTANCE);
        }
        if self.program_mode == ProgramMode::Menu {
            if let Some(digit) = input.pressed_digit() {
                let body = digit + 1;
                if body < self.system.body_count() {
                    self.destination = body;
                    self.notify(format!(
                        "DESTINO: {} - PULSA F PARA WARP",
                        self.body_name(body)
                    ));
                }
            }
            if input.was_pressed(Key::Tab) {
                self.destination = (self.destination + 1) % self.system.body_count();
                self.notify(format!(
                    "DESTINO: {} - PULSA F PARA WARP",
                    self.body_name(self.destination)
                ));
            }
            if input.was_pressed(Key::F) {
                self.start_warp(effect);
            }
        }
    }

    fn handle_seed_commands(&mut self, input: &InputState) -> bool {
        if self.program_mode != ProgramMode::Menu {
            return false;
        }

        if let Some(mut value) = self.seed_input.take() {
            if input.was_pressed(Key::Escape) {
                self.notify("EDICION DE SEMILLA CANCELADA");
                return true;
            }
            if input.was_pressed(Key::Backspace) {
                value.pop();
            }
            if let Some(digit) = input.pressed_numeric_digit() {
                let candidate = format!("{}{}", value, digit);
                if candidate.len() <= 10 && candidate.parse::<u32>().is_ok() {
                    value = candidate;
                }
            }
            if input.was_pressed(Key::Enter) {
                if let Ok(seed) = value.parse::<u32>() {
                    self.next_run_seed = seed;
                    self.notify(format!("SEMILLA SELECCIONADA: {}", seed));
                } else {
                    self.notify("ESCRIBE AL MENOS UN DIGITO");
                    self.seed_input = Some(value);
                }
                return true;
            }
            self.seed_input = Some(value);
            return true;
        }

        if input.was_pressed(Key::I) {
            self.ship.stop();
            self.seed_input = Some(String::new());
            self.notify("ESCRIBE LA SEMILLA Y PULSA ENTER");
            return true;
        }
        if input.was_pressed(Key::R) {
            self.next_run_seed = generate_seed();
            self.notify(format!("NUEVA SEMILLA: {}", self.next_run_seed));
            return true;
        }
        false
    }

    fn update_menu_orbit_lockout(&mut self) {
        let Some(body) = self.menu_orbit_lockout else {
            return;
        };
        let distance = (self.system.position(body) - self.ship.position).length();
        let radius = self.system.body(body).orbit_capture_radius;
        if !Ship::can_orbit(distance, radius) {
            self.menu_orbit_lockout = None;
        }
    }

    fn try_enter_menu_orbit(&mut self) {
        if self.program_mode != ProgramMode::Menu
            || self.ship.is_orbiting()
            || self.warp.is_traveling()
        {
            return;
        }
        let nearest = (1..self.system.body_count())
            .map(|body| {
                let distance = (self.system.position(body) - self.ship.position).length();
                (body, distance, self.system.body(body).orbit_capture_radius)
            })
            .filter(|&(body, distance, radius)| {
                self.menu_orbit_lockout != Some(body) && Ship::can_orbit(distance, radius)
            })
            .min_by(|a, b| (a.1 / a.2).total_cmp(&(b.1 / b.2)));
        if let Some((body, _, _)) = nearest {
            let radius = self.system.body(body).radius;
            self.ship
                .enter_orbit(body, self.system.position(body), radius);
            self.notify(format!(
                "ORBITA AUTOMATICA: {} - A/D GIRAR, W ACERCAR, S ALEJAR/SALIR",
                self.body_name(body)
            ));
        }
    }

    fn try_start_game_transition(&mut self) {
        if self.program_mode != ProgramMode::Menu
            || !matches!(self.start_transition, StartTransition::Idle)
        {
            return;
        }
        let sun_distance = (self.ship.position - self.system.sun_position()).length();
        if sun_distance <= self.system.body(0).radius * SUN_ENTRY_DEPTH {
            self.ship.stop();
            self.notice = None;
            self.start_transition = StartTransition::Flashing {
                elapsed: 0.0,
                switched: false,
            };
        }
    }

    fn update_start_transition(&mut self, delta_time: f32, camera: &mut Camera) {
        let StartTransition::Flashing {
            mut elapsed,
            mut switched,
        } = self.start_transition
        else {
            return;
        };
        elapsed += delta_time;
        if !switched && elapsed >= START_FLASH_IN_DURATION {
            self.configure_mode(ProgramMode::Game, camera);
            switched = true;
        }
        self.start_transition = if elapsed >= START_FLASH_DURATION {
            StartTransition::Idle
        } else {
            StartTransition::Flashing { elapsed, switched }
        };
    }

    fn start_warp(&mut self, effect: &mut WarpEffect) {
        let target = self.destination;
        let distance = (self.system.position(target) - self.ship.position).length();
        if distance <= warp::safe_distance(self.system.body(target).radius) + 1.0 {
            self.notify(format!("YA ESTAS EN {}", self.body_name(target)));
            return;
        }
        if self.warp.engage(target, &mut self.ship, effect) {
            self.notify(format!("WARP HACIA {}", self.body_name(target)));
        } else if !self.warp.is_traveling() {
            self.notify("WARP RECARGANDO");
        }
    }

    fn fly(&mut self, controls: &ShipControls, delta_time: f32) {
        match self.ship.mode {
            FlightMode::Free => self.ship.update_free(controls, delta_time),
            FlightMode::Orbit { body, .. } => {
                let center = self.system.position(body);
                let displacement = self.system.displacement(body);
                let radius = self.system.body(body).radius;
                let capture_radius = self.system.body(body).orbit_capture_radius;
                if self.ship.update_orbit(
                    controls,
                    center,
                    displacement,
                    radius,
                    capture_radius,
                    delta_time,
                ) {
                    if self.program_mode == ProgramMode::Menu {
                        self.menu_orbit_lockout = Some(body);
                    }
                    self.notify("ORBITA LIBERADA - VUELO LIBRE");
                }
            }
        }
    }

    fn follow_ship(&self, camera: &mut Camera, delta_time: f32) {
        let (forward, up, distance, height, look_ahead, smoothing) = match self.ship.orbit_body() {
            Some(body) => {
                let to_center = self.system.position(body) - self.ship.position;
                let orbit_forward = to_center.normalize();
                let transition = self.ship.orbit_transition();
                let eased = transition * transition * (3.0 - 2.0 * transition);
                let orbit_up = if orbit_forward.y.abs() > 0.92 {
                    Vec3::new(1.0, 0.0, 0.0)
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };
                (
                    (self.ship.forward() * (1.0 - eased) + orbit_forward * eased).normalize(),
                    (self.ship.up() * (1.0 - eased) + orbit_up * eased).normalize(),
                    self.follow_distance * (1.0 + (ORBIT_CAMERA_DISTANCE_FACTOR - 1.0) * eased),
                    self.follow_distance
                        * (FOLLOW_HEIGHT_RATIO * (1.0 - eased) + ORBIT_CAMERA_HEIGHT_RATIO * eased),
                    FOLLOW_LOOK_AHEAD + (to_center.length() - FOLLOW_LOOK_AHEAD) * eased,
                    FOLLOW_SMOOTHING + (ORBIT_CAMERA_SMOOTHING - FOLLOW_SMOOTHING) * eased,
                )
            }
            None => (
                self.ship.forward(),
                self.ship.up(),
                self.follow_distance,
                self.follow_distance * FOLLOW_HEIGHT_RATIO,
                FOLLOW_LOOK_AHEAD,
                FOLLOW_SMOOTHING,
            ),
        };
        camera.follow(
            self.ship.position,
            forward,
            up,
            distance,
            height,
            look_ahead,
            smoothing,
            delta_time,
        );
    }

    fn fly_free_camera(&self, camera: &mut Camera, controls: &ShipControls, delta_time: f32) {
        camera.rotate(
            controls.yaw * FREE_CAMERA_YAW_RATE * delta_time,
            controls.pitch * FREE_CAMERA_PITCH_RATE * delta_time,
            controls.roll * FREE_CAMERA_ROLL_RATE * delta_time,
        );
        let speed = if controls.boost {
            FREE_CAMERA_BOOST
        } else {
            FREE_CAMERA_SPEED
        } * delta_time;
        camera.move_local(controls.thrust * speed, 0.0, controls.vertical * speed);
    }

    pub fn populate(&self, scene: &mut Scene, assets: &AssetManager) {
        scene.clear();
        for index in 0..self.system.body_count() {
            let body = self.system.body(index);
            let state = self.system.state(index);
            let model = self.models.bodies[index];
            let unit_radius = match model {
                Model::Lod(handle) => assets.lod_bounding_radius(handle),
                Model::Mesh(handle) => assets.mesh_bounding_radius(handle),
            };
            scene.push(Entity {
                model,
                material: body.material,
                transform: Transform::new(state.position)
                    .with_rotation(state.rotation)
                    .with_uniform_scale(body.radius),
                radius: body.radius * unit_radius.max(1.0),
            });
        }

        for surface in &self.surfaces {
            let body = self.system.body(surface.body);
            let state = self.system.state(surface.body);
            let surface_radius = body.radius * (1.0 - surface.embed_depth);
            for prop in &surface.props {
                let position =
                    state.position + rotate_vector(state.rotation, prop.direction * surface_radius);
                let (model, material, scale) = match prop.kind {
                    PropKind::Crystal => (self.models.crystal, self.materials.crystal, prop.scale),
                    PropKind::Boulder => (
                        self.models.boulder,
                        self.materials.boulder,
                        prop.scale * 0.6,
                    ),
                };
                scene.push(Entity {
                    model,
                    material,
                    transform: Transform::new(position)
                        .with_rotation(state.rotation * prop.local_rotation())
                        .with_uniform_scale(scale),
                    radius: scale * 1.4,
                });
            }
        }

        let orbital_time = self.system.orbital_time();
        let system_origin = self.system.origin();
        for asteroid in &self.asteroids {
            scene.push(Entity {
                model: self.models.asteroid,
                material: self.materials.asteroid,
                transform: Transform::new(system_origin + asteroid.position(orbital_time))
                    .with_rotation(asteroid.rotation(self.time))
                    .with_scale(asteroid.scale),
                radius: asteroid.radius() * 1.4,
            });
        }

        for enemy in &self.menu_patrols {
            let host = self.system.position(enemy.host);
            let forward = enemy.orbit.direction_of_travel(self.time);
            let up = enemy.orbit.offset(self.time).normalize();
            scene.push(Entity {
                model: Model::Mesh(self.models.enemy),
                material: self.materials.enemy,
                transform: Transform::new(host + enemy.orbit.offset(self.time))
                    .with_rotation(
                        basis_rotation(forward, up) * Mat4::rotation_y(MODEL_FORWARD_CORRECTION),
                    )
                    .with_uniform_scale(enemy.scale),
                radius: enemy.scale,
            });
        }

        for enemy in self.run_enemies.enemies() {
            let forward = if enemy.velocity.length() > f32::EPSILON {
                enemy.velocity.normalize()
            } else {
                (self.ship.position - enemy.position).normalize()
            };
            scene.push(Entity {
                model: Model::Mesh(self.models.enemy),
                material: self.materials.enemy,
                transform: Transform::new(enemy.position)
                    .with_rotation(
                        basis_rotation(forward, Vec3::new(0.0, 1.0, 0.0))
                            * Mat4::rotation_y(MODEL_FORWARD_CORRECTION),
                    )
                    .with_uniform_scale(ENEMY_RENDER_SCALE),
                radius: ENEMY_RENDER_SCALE,
            });
        }

        scene.push(Entity {
            model: Model::Mesh(self.models.ship),
            material: self.materials.ship,
            transform: Transform::new(self.ship.position)
                .with_rotation(self.ship.rotation() * Mat4::rotation_y(MODEL_FORWARD_CORRECTION))
                .with_uniform_scale(SHIP_RADIUS),
            radius: SHIP_RADIUS,
        });
    }

    pub fn queue_lines(&self, renderer: &mut Renderer) {
        for index in 0..self.system.body_count() {
            let Some(path) = self.system.orbit_path(index, ORBIT_SEGMENTS) else {
                continue;
            };
            let color = if index == self.destination {
                DESTINATION_COLOR
            } else {
                ORBIT_COLOR
            };
            for segment in path.windows(2).step_by(2) {
                renderer.draw_line_3d(segment[0], segment[1], color, false);
            }
        }
        for projectile in self.weapons.projectiles() {
            renderer.draw_line_3d(
                projectile.line_start(),
                projectile.position,
                PROJECTILE_COLOR,
                false,
            );
        }
    }

    pub fn hud_info(&self) -> HudInfo {
        let flight_mode = if self.warp.is_traveling() {
            "WARP".to_owned()
        } else {
            match self.ship.mode {
                FlightMode::Free => "LIBRE".to_owned(),
                FlightMode::Orbit { body, .. } => format!("ORBITA {}", self.body_name(body)),
            }
        };
        let destination_center = self.system.position(self.destination);
        HudInfo {
            program_mode: match self.program_mode {
                ProgramMode::Menu => "MENU",
                ProgramMode::Game => "JUEGO",
            },
            objective: match self.program_mode {
                ProgramMode::Menu => Some("OBJETIVO: VUELA DENTRO DEL SOL PARA INICIAR"),
                ProgramMode::Game => match self.session.state() {
                    RunState::SolarCollapse => Some("COLAPSO SOLAR"),
                    RunState::PlayerDestroyed => Some("NAVE DESTRUIDA"),
                    _ => None,
                },
            },
            run_time_remaining: (self.program_mode == ProgramMode::Game)
                .then(|| self.session.remaining()),
            run_seed: if self.program_mode == ProgramMode::Game {
                self.session.seed()
            } else {
                self.next_run_seed
            },
            seed_input: self.seed_input.clone(),
            progression: (self.program_mode == ProgramMode::Game).then(|| hud::ProgressHudInfo {
                health: self.progression.health(),
                max_health: self.progression.max_health(),
                health_ratio: self.progression.health_ratio(),
                level: self.progression.level(),
                experience: self.progression.experience(),
                experience_required: self.progression.experience_required(),
                experience_ratio: self.progression.experience_ratio(),
            }),
            speed: self.ship.speed(),
            flight_mode,
            camera_mode: match self.camera_mode {
                CameraMode::Follow => "SEGUIR",
                CameraMode::Free => "LIBRE",
            },
            destination_number: self.destination,
            destination_name: self.body_name(self.destination),
            destination_distance: (destination_center - self.ship.position).length()
                - self.system.body(self.destination).radius,
            warp_status: match self.warp.state() {
                WarpState::Ready => "LISTO",
                WarpState::Traveling { .. } => "ACTIVO",
                WarpState::Cooldown { .. } => "RECARGA",
            },
            notice: self.notice.as_ref().map(|(message, _)| message.clone()),
            show_help: self.show_help,
        }
    }

    pub fn light_position(&self) -> Vec3 {
        self.system.sun_position()
    }

    pub fn transition_flash(&self) -> f32 {
        match self.start_transition {
            StartTransition::Idle => 0.0,
            StartTransition::Flashing { elapsed, .. } => {
                let amount = if elapsed < START_FLASH_IN_DURATION {
                    elapsed / START_FLASH_IN_DURATION
                } else if elapsed < START_FLASH_IN_DURATION + START_FLASH_HOLD_DURATION {
                    1.0
                } else {
                    1.0 - (elapsed - START_FLASH_IN_DURATION - START_FLASH_HOLD_DURATION)
                        / START_FLASH_OUT_DURATION
                }
                .clamp(0.0, 1.0);
                amount * amount * (3.0 - 2.0 * amount)
            }
        }
    }
}

pub fn ship_controls(input: &InputState) -> ShipControls {
    let mouse = if input.is_button_held(MouseButton::Right) {
        input.mouse_delta() * MOUSE_SENSITIVITY
    } else {
        Default::default()
    };
    ShipControls {
        thrust: input.axis(Key::W, Key::S),
        vertical: input.axis(Key::Space, Key::LeftControl),
        yaw: (input.axis(Key::A, Key::D) + input.axis(Key::Left, Key::Right) - mouse.x)
            .clamp(-1.0, 1.0),
        pitch: (input.axis(Key::Up, Key::Down) - mouse.y).clamp(-1.0, 1.0),
        roll: input.axis(Key::E, Key::Q),
        boost: input.is_held(Key::LeftShift),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_game() -> (Game, Camera) {
        let mut assets = AssetManager::new();
        let mut camera = Camera::new(Vec3::default(), 16.0 / 9.0);
        let game = Game::new(&mut assets, &mut camera);
        (game, camera)
    }

    #[test]
    fn game_starts_in_menu_and_sun_flash_switches_to_game() {
        let (mut game, mut camera) = test_game();
        let input = InputState::new();
        let mut effect = WarpEffect::new();
        assert_eq!(game.program_mode, ProgramMode::Menu);
        assert_eq!(
            game.hud_info().objective,
            Some("OBJETIVO: VUELA DENTRO DEL SOL PARA INICIAR")
        );

        game.ship.position = game.system.sun_position();
        game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);
        assert!(matches!(
            game.start_transition,
            StartTransition::Flashing { .. }
        ));

        game.update(&input, START_FLASH_IN_DURATION, &mut camera, &mut effect);
        assert_eq!(game.program_mode, ProgramMode::Game);
        assert_eq!(game.session.state(), RunState::Playing);
        assert_ne!(game.session.seed(), 0);
        assert_eq!(camera.far, CAMERA_FAR);
        assert_eq!(game.hud_info().objective, None);
        assert!((game.transition_flash() - 1.0).abs() < 0.001);
        assert_eq!(game.system.body_count(), 1);
        assert_eq!(game.system.body(0).name, "SOL");
    }

    #[test]
    fn menu_planet_orbit_is_automatic() {
        let (mut game, mut camera) = test_game();
        let input = InputState::new();
        let mut effect = WarpEffect::new();
        let body = 1;
        let center = game.system.position(body);
        let radius = game.system.body(body).radius;
        game.ship.position = center + Vec3::new(radius * 2.0, 0.0, 0.0);

        game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);
        assert_eq!(game.ship.orbit_body(), Some(body));
        assert_eq!(game.ship.orbit_transition(), 0.0);

        for _ in 0..240 {
            game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);
        }
        let toward_planet = (game.system.position(body) - camera.position).normalize();
        assert!(camera.forward().dot(toward_planet) > 0.99);
    }

    #[test]
    fn selected_seed_is_kept_by_the_run_session() {
        let (mut first, mut first_camera) = test_game();
        let (mut second, mut second_camera) = test_game();
        first.next_run_seed = 12_345;
        second.next_run_seed = 12_345;

        first.configure_mode(ProgramMode::Game, &mut first_camera);
        second.configure_mode(ProgramMode::Game, &mut second_camera);

        assert_eq!(first.session.seed(), 12_345);
        assert_eq!(first.session.seed(), second.session.seed());
        assert!(first.asteroids.is_empty());
        assert!(first.surfaces.is_empty());
        assert!(first.menu_patrols.is_empty());
        assert_eq!(first.run_enemies.active_count(), 0);
    }

    #[test]
    fn player_damage_ends_the_run_and_progress_resets_on_next_run() {
        let (mut game, mut camera) = test_game();
        game.configure_mode(ProgramMode::Game, &mut camera);
        game.grant_player_experience(25);

        assert_eq!(game.progression.level(), 2);
        assert!(!game.damage_player(99.0));
        assert!(game.damage_player(1.0));
        assert_eq!(game.session.state(), RunState::PlayerDestroyed);

        game.configure_mode(ProgramMode::Game, &mut camera);
        assert_eq!(game.progression, PlayerProgression::new());
        assert_eq!(game.session.state(), RunState::Playing);
    }

    #[test]
    fn run_enemy_contact_is_connected_to_player_health() {
        let (mut game, mut camera) = test_game();
        let input = InputState::new();
        let mut effect = WarpEffect::new();
        game.configure_mode(ProgramMode::Game, &mut camera);
        game.run_enemies
            .spawn_at(game.ship.position, game.progression.level());

        game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);

        assert_eq!(game.progression.health(), 90.0);
        assert_eq!(game.session.state(), RunState::Playing);
    }
}
