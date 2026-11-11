pub mod buffs;
pub mod effects;
pub mod enemies;
pub mod enemy_beams;
pub mod hud;
pub mod player;
pub mod progression;
pub mod roulette;
pub mod run_world;
pub mod session;
pub mod sun_lifecycle;
pub mod upgrades;
pub mod warp;
pub mod wave_director;
pub mod weapons;

use crate::assets::material::{glowing, solid};
use crate::assets::mesh::{crystal, placeholder_ship, turned_around};
use crate::assets::{AssetManager, MeshHandle};
use crate::engine::camera::Camera;
use crate::engine::collision::{resolve_all, Collider};
use crate::engine::gravity::{total_acceleration, GravityWell};
use crate::engine::math::{Mat4, Vec3};
use crate::engine::scene::{Entity, Model, Scene};
use crate::engine::transform::{basis_rotation, rotate_vector, Transform};
use crate::platform::audio::SoundEffect;
use crate::platform::input::{InputState, Key, MouseButton};
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use crate::renderer::shaders::ShaderKind;
use crate::renderer::warp_effect::WarpEffect;
use crate::renderer::{Deformation, Material, Mesh, Renderer};
use crate::world::generation::{asteroid_belt, scatter_surface, Asteroid, PropKind, SurfaceProp};
use crate::world::orbit::Orbit;
use crate::world::solar_system::MAX_TIME_SCALE;
use crate::world::{SolarSystem, SystemConfig};
use buffs::{ActiveBuffs, BuffKind};
use effects::Effects;
use roulette::Roulette;
use upgrades::{apply_option, planet_reward, upgrade_options, Reward};
use crate::platform::window::TextLabel;
use crate::world::generation::SeededRandom;
use crate::renderer::culling::Frustum;
use enemies::{EnemyKind, EnemySwarm, PlayerMotion, ENEMY_RENDER_SCALE};
use enemy_beams::{BeamOwner, BeamRequest, EnemyBeams};
use hud::HudInfo;
use player::{FlightMode, Ship, ShipControls, SHIP_RADIUS};
use progression::PlayerProgression;
use run_world::{DestroyedPlanet, PlanetKind, RunWorld};
use session::{generate_seed, RunSession, RunState};
use std::f32::consts::PI;
use sun_lifecycle::{follow_offset, radius_multiplier};
use warp::{Destination, Warp, WarpEvent, WarpState};
use wave_director::WaveDirector;
use weapons::{Aim, CombatEvents, Modifiers, Targets, WeaponSystem};

const GAME_FLIGHT_SPEED_MULTIPLIER: f32 = 1.5;
const SPHERE_SUBDIVISIONS: [u32; 3] = [4, 3, 1];
const ENEMY_VISIBILITY_RADIUS: f32 = 3.0;
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
const EVASIVE_LOOP_CAMERA_SMOOTHING: f32 = 1.5;
const CROSSHAIR_PREVIEW_DISTANCE: f32 = 45.0;
const CROSSHAIR_SCREEN_MARGIN: i32 = 6;
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
const RESISTANT_COLOR: Vec3 = Vec3::new(0.32, 0.36, 0.62);
const ELITE_COLOR: Vec3 = Vec3::new(0.95, 0.12, 0.10);
const ELITE_GLOW: Vec3 = Vec3::new(0.18, 0.02, 0.0);
const ELITE_BAR_COLOR: Color = Color::new(255, 60, 50);
const RESISTANT_BAR_COLOR: Color = Color::new(150, 160, 255);
const ENEMY_BAR_BACKGROUND: Color = Color::new(70, 20, 20);
const MAX_BEAM_IMPACT_EFFECTS: usize = 6;
const MAX_IMPACT_EFFECTS: usize = 8;
const DEATH_DELAY: f32 = 2.5;
const SUMMARY_AUTO_RETURN: f32 = 20.0;
const DAMAGE_NUMBER_LIFE: f32 = 0.8;
const DAMAGE_NUMBER_RISE: f32 = 3.0;
const DAMAGE_NUMBER_MERGE: f32 = 2.0;
const MAX_DAMAGE_NUMBERS: usize = 40;
const DAMAGE_NUMBER_COLOR: Color = Color::new(255, 230, 120);
const DAMAGE_NUMBER_SIZE: f32 = 6.0;
const GAS_RING_COLOR: Color = Color::new(150, 205, 255);
const GAS_RING_DIM: Color = Color::new(70, 110, 160);
const DROID_COLOR: Color = Color::new(80, 230, 255);
const DROID_SATELLITES: usize = 3;
const CORE_HALO_COLOR: Color = Color::new(255, 130, 50);
const CORE_GLOW: Vec3 = Vec3::new(0.25, 0.08, 0.0);
const ROCKY_DEBRIS: usize = 7;
const ROCKY_DEBRIS_COLOR: Color = Color::new(217, 194, 154);
const COLLAPSE_DURATION: f32 = 8.0;
const COLLAPSE_PULL: f32 = 40.0;
const COLLAPSE_MIN_SUN: f32 = 0.08;
const DAMAGE_FLASH_DURATION: f32 = 0.35;
const DAMAGE_FLASH_STRENGTH: f32 = 0.45;
const INVULNERABILITY_DURATION: f32 = 0.75;
const BLINK_RATE: f32 = 22.0;
const LAST_MINUTE: f32 = 60.0;
const TARGET_CONE: f32 = 0.995;
const TARGET_RANGE: f32 = 260.0;
const SHIELD_COLOR: Color = Color::new(90, 230, 255);
const DAMAGE_COLOR: Color = Color::new(255, 30, 30);
const ORBIT_COLOR: Color = Color::new(192, 80, 255);
const DESTINATION_COLOR: Color = Color::new(92, 255, 92);
pub const START_POSITION: Vec3 = Vec3::new(0.0, 10.0, 104.0);
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
const ENEMY_GLOW: Vec3 = Vec3::new(0.30, 0.28, 0.35);
const SAND_COLOR: Vec3 = Vec3::new(0.85, 0.76, 0.6);
const CRYSTAL_COLOR: Vec3 = Vec3::new(0.75, 0.31, 1.0);
const CRYSTAL_GLOW: Vec3 = Vec3::new(0.12, 0.03, 0.18);
const BOULDER_COLOR: Vec3 = Vec3::new(0.62, 0.5, 0.38);

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
    run_planets: [Model; PlanetKind::COUNT],
}

struct Materials {
    ship: Material,
    enemy: Material,
    asteroid: Material,
    crystal: Material,
    boulder: Material,
    run_planets: [Material; PlanetKind::COUNT],
}

struct Surface {
    body: usize,
    props: Vec<SurfaceProp>,
    embed_depth: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunStats {
    pub enemies_destroyed: u32,
    pub planets_destroyed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DamageNumber {
    position: Vec3,
    amount: f32,
    life: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunSummary {
    pub cause: &'static str,
    pub survived: f32,
    pub level: u32,
    pub enemies_destroyed: u32,
    pub planets_destroyed: u32,
    pub weapons: Vec<String>,
    pub seed: u32,
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
    pub gravity_enabled: bool,
    asteroids: Vec<Asteroid>,
    surfaces: Vec<Surface>,
    menu_patrols: Vec<PatrolEnemy>,
    run_enemies: EnemySwarm,
    wave_director: WaveDirector,
    weapons: WeaponSystem,
    run_world: RunWorld,
    effects: Effects,
    run_stats: RunStats,
    enemy_beams: EnemyBeams,
    buffs: ActiveBuffs,
    reward_random: SeededRandom,
    pending_upgrades: u32,
    roulette: Option<Roulette>,
    ending_timer: f32,
    ending_cause: Option<&'static str>,
    summary: Option<RunSummary>,
    ship_absorbed: bool,
    damage_flash: f32,
    invulnerable_timer: f32,
    last_minute_warned: bool,
    sounds: Vec<SoundEffect>,
    damage_numbers: Vec<DamageNumber>,
    planet_collider_start: usize,
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
    let bodies: Vec<Model> = (0..system.body_count())
        .map(|index| match system.body(index).model.file_name() {
            Some(file_name) => Model::Mesh(
                assets.load_model_or_else(file_name, || Mesh::icosphere(SPHERE_SUBDIVISIONS[0])),
            ),
            None => sphere,
        })
        .collect();
    let run_planets = [
        bodies[system.find("ROCOSO").unwrap()],
        bodies[system.find("GASEOSO").unwrap()],
        bodies[system.find("NUCLEO").unwrap()],
        bodies[system.find("DROIDE").unwrap()],
    ];
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
        run_planets,
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
        let run_planet_materials = [
            system.body(system.find("ROCOSO").unwrap()).material,
            system.body(system.find("GASEOSO").unwrap()).material,
            system.body(system.find("NUCLEO").unwrap()).material,
            system.body(system.find("DROIDE").unwrap()).material,
        ];
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
            gravity_enabled: true,
            asteroids,
            surfaces,
            menu_patrols,
            run_enemies: EnemySwarm::new(),
            wave_director: WaveDirector::new(),
            weapons: WeaponSystem::new(),
            run_world: RunWorld::new(),
            effects: Effects::new(),
            run_stats: RunStats::default(),
            enemy_beams: EnemyBeams::new(),
            buffs: ActiveBuffs::new(),
            reward_random: SeededRandom::new(1),
            pending_upgrades: 0,
            roulette: None,
            ending_timer: 0.0,
            ending_cause: None,
            summary: None,
            ship_absorbed: false,
            damage_flash: 0.0,
            invulnerable_timer: 0.0,
            last_minute_warned: false,
            sounds: Vec::new(),
            damage_numbers: Vec::new(),
            planet_collider_start: 0,
            models,
            materials: Materials {
                ship: glowing(PLAYER_COLOR, PLAYER_GLOW, ShaderKind::EtaPlayer),
                enemy: glowing(ENEMY_COLOR, ENEMY_GLOW, ShaderKind::EnemyShip),
                asteroid: solid(SAND_COLOR, ShaderKind::Rocky),
                crystal: glowing(CRYSTAL_COLOR, CRYSTAL_GLOW, ShaderKind::Lambert),
                boulder: solid(BOULDER_COLOR, ShaderKind::Rocky),
                run_planets: run_planet_materials,
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
        self.effects.clear();
        self.run_stats = RunStats::default();
        self.enemy_beams.clear();
        self.buffs.reset();
        self.pending_upgrades = 0;
        self.roulette = None;
        self.ending_timer = 0.0;
        self.ending_cause = None;
        self.summary = None;
        self.ship_absorbed = false;
        self.damage_flash = 0.0;
        self.invulnerable_timer = 0.0;
        self.last_minute_warned = false;
        self.damage_numbers.clear();
        self.reward_random = SeededRandom::new(self.next_run_seed ^ 0x0BAD_5EED);
        match mode {
            ProgramMode::Menu => {
                self.session.reset();
                self.run_enemies.clear();
                self.wave_director.reset();
                self.weapons.reset();
                self.run_world.clear();
            }
            ProgramMode::Game => {
                self.session.start(self.next_run_seed);
                self.run_enemies.reset(self.next_run_seed);
                self.wave_director.reset();
                self.weapons.reset();
                self.run_world.reset(self.next_run_seed);
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
        self.ship.speed_multiplier = flight_speed_multiplier(
            self.program_mode,
            self.buffs.multiplier(BuffKind::ShipSpeed),
        );
        self.update_evasive_loop_input(input);
        self.update_upgrades(input, delta_time);
        self.update_ending(input, delta_time, camera);
        if self.program_mode == ProgramMode::Game && self.session.update(delta_time) {
            self.ship.stop();
            self.ending_timer = 0.0;
            self.ending_cause = Some("EL SOL COLAPSO");
            self.sounds.push(SoundEffect::Collapse);
            self.notify("EL SOL HA COMENZADO A COLAPSAR");
        }
        self.update_sun_lifecycle(delta_time);
        self.system.update(delta_time);
        if self.program_mode == ProgramMode::Game {
            self.run_world.update(self.ship.position);
        }
        self.refresh_colliders();

        let run_is_paused = self.program_mode == ProgramMode::Game
            && !matches!(
                self.session.state(),
                RunState::Playing | RunState::SolarCollapse
            );
        let controls = if self.seed_input.is_some() || run_is_paused {
            ShipControls::default()
        } else if self.session.state() == RunState::SolarCollapse {
            scaled_controls(ship_controls(input), 1.0 - self.collapse_progress())
        } else {
            ship_controls(input)
        };
        let ship_controls = if self.camera_mode == CameraMode::Free {
            ShipControls::default()
        } else {
            controls
        };
        if self.simulation_frozen() {
        } else if self.warp.is_traveling() {
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
            self.buffs.update(delta_time);
        }
        if self.program_mode == ProgramMode::Game && self.session.state() == RunState::Playing {
            let frustum = Frustum::from_view_projection(camera.view_projection_matrix());
            let is_visible =
                |position: Vec3| frustum.contains_sphere(position, ENEMY_VISIBILITY_RADIUS);
            let motion = PlayerMotion {
                position: self.ship.position,
                velocity: self.ship.velocity,
                forward: self.ship.forward(),
            };
            self.run_enemies.set_run_elapsed(self.session.elapsed());
            let capacity = self.wave_director.settings(self.session.elapsed()).max_active;
            let recycled = self.run_enemies.recycle_stragglers(
                motion,
                &self.colliders,
                is_visible,
                capacity,
                delta_time,
            );
            for position in recycled {
                self.effects.enemy_arrival(position);
            }
            let spawn_count = self.wave_director.update(
                self.session.elapsed(),
                self.run_enemies.engaged_count(),
                delta_time,
            );
            for _ in 0..spawn_count {
                let kind = self.wave_director.choose_kind(
                    self.session.elapsed(),
                    self.run_enemies.count_kind(EnemyKind::Elite),
                    self.reward_random.next(),
                );
                let id = self.run_enemies.spawn_kind_around(
                    motion,
                    self.progression.level(),
                    kind,
                    &self.colliders,
                    is_visible,
                );
                if kind == EnemyKind::Elite {
                    self.notify("NAVE ELITE EN CAMINO");
                }
                if let Some(position) = self.run_enemies.position_of(id) {
                    self.effects.enemy_arrival(position);
                }
            }
            let aim = self.aim();
            let modifiers = self.weapon_modifiers();
            let mut targets = Targets {
                enemies: &mut self.run_enemies,
                planets: &mut self.run_world,
                obstacles: &self.colliders[..self.planet_collider_start],
            };
            let events = self
                .weapons
                .update(aim, &mut targets, modifiers, delta_time);
            self.handle_combat_events(events);
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
            let player_position = self.ship.position;
            let beams = &mut self.enemy_beams;
            let colliders = &self.colliders;
            self.run_enemies
                .elite_shots(player_position, delta_time, |id, origin| {
                    beams.request(
                        BeamRequest {
                            owner: BeamOwner::Enemy(id),
                            origin,
                            target: player_position,
                        },
                        colliders,
                    )
                });
            let beam_events = self.enemy_beams.update(
                delta_time,
                player_position,
                SHIP_RADIUS,
                &mut self.run_enemies,
            );
            self.handle_beam_events(beam_events);
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

        self.effects.update(delta_time);
        self.update_damage_numbers(delta_time);
        effect.update(delta_time);
        if let Some((_, remaining)) = &mut self.notice {
            *remaining -= delta_time;
            if *remaining <= 0.0 {
                self.notice = None;
            }
        }
    }

    fn simulation_frozen(&self) -> bool {
        self.program_mode == ProgramMode::Game
            && matches!(
                self.session.state(),
                RunState::UpgradeRoulette | RunState::PlayerDestroyed | RunState::Summary
            )
    }

    fn collapse_progress(&self) -> f32 {
        if self.ending_cause == Some("EL SOL COLAPSO") {
            (self.ending_timer / COLLAPSE_DURATION).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn update_ending(&mut self, input: &InputState, delta_time: f32, camera: &mut Camera) {
        if self.program_mode != ProgramMode::Game {
            return;
        }
        self.damage_flash = (self.damage_flash - delta_time).max(0.0);
        self.invulnerable_timer = (self.invulnerable_timer - delta_time).max(0.0);
        if !self.last_minute_warned
            && self.session.state() == RunState::Playing
            && self.session.remaining() <= LAST_MINUTE
        {
            self.last_minute_warned = true;
            self.notify("ULTIMO MINUTO - EL SOL ESTA POR COLAPSAR");
            self.sounds.push(SoundEffect::Alarm);
        }
        match self.session.state() {
            RunState::PlayerDestroyed => {
                self.ending_timer += delta_time;
                if self.ending_timer >= DEATH_DELAY {
                    self.finish_run();
                }
            }
            RunState::SolarCollapse => {
                self.ending_timer += delta_time;
                let progress = self.collapse_progress();
                let sun = self.system.sun_position();
                let to_sun = sun - self.ship.position;
                let distance = to_sun.length();
                if distance > f32::EPSILON {
                    self.ship.velocity = self.ship.velocity
                        + to_sun * (COLLAPSE_PULL * progress * progress * delta_time / distance);
                }
                let swallow_radius = self.system.body(0).radius * 1.6 + 6.0 * progress;
                if !self.ship_absorbed && progress > 0.6 && distance < swallow_radius {
                    self.ship_absorbed = true;
                    self.effects.player_explosion(self.ship.position);
                }
                if progress >= 1.0 {
                    self.finish_run();
                }
            }
            RunState::Summary => {
                self.ending_timer += delta_time;
                if input.was_pressed(Key::Enter) || self.ending_timer >= SUMMARY_AUTO_RETURN {
                    self.configure_mode(ProgramMode::Menu, camera);
                } else if input.was_pressed(Key::R) {
                    self.next_run_seed = self.session.seed();
                    self.configure_mode(ProgramMode::Game, camera);
                }
            }
            _ => {}
        }
    }

    fn finish_run(&mut self) {
        self.summary = Some(RunSummary {
            cause: self.ending_cause.unwrap_or("PARTIDA TERMINADA"),
            survived: self.session.elapsed(),
            level: self.progression.level(),
            enemies_destroyed: self.run_stats.enemies_destroyed,
            planets_destroyed: self.run_stats.planets_destroyed,
            weapons: self
                .weapons
                .inventory
                .owned()
                .map(|(kind, level)| format!("{} NV{}", kind.name(), level))
                .collect(),
            seed: self.session.seed(),
        });
        self.roulette = None;
        self.ending_timer = 0.0;
        self.session.show_summary();
    }

    pub fn summary(&self) -> Option<&RunSummary> {
        self.summary.as_ref()
    }

    pub fn black_hole(&self) -> (Vec3, f32) {
        let progress = self.collapse_progress();
        let intensity = ((progress - 0.15) / 0.6).clamp(0.0, 1.0);
        (self.system.sun_position(), intensity * intensity * (3.0 - 2.0 * intensity))
    }

    pub fn damage_flash(&self) -> (Color, f32) {
        let amount = self.damage_flash / DAMAGE_FLASH_DURATION * DAMAGE_FLASH_STRENGTH;
        (DAMAGE_COLOR, amount.clamp(0.0, 1.0))
    }

    pub fn crosshair_screen_position(&self, camera: &Camera) -> Option<(i32, i32)> {
        if !self.ship_visible() {
            return None;
        }
        let target = self.ship.position + self.ship.forward() * CROSSHAIR_PREVIEW_DISTANCE;
        let clip = camera.view_projection_matrix() * target.extend(1.0);
        if clip.w <= 0.001 {
            return None;
        }
        let x = (clip.x / clip.w + 1.0) * 0.5 * FRAMEBUFFER_WIDTH as f32;
        let y = (1.0 - clip.y / clip.w) * 0.5 * FRAMEBUFFER_HEIGHT as f32;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let max_x = FRAMEBUFFER_WIDTH as i32 - 1 - CROSSHAIR_SCREEN_MARGIN;
        let max_y = FRAMEBUFFER_HEIGHT as i32 - 1 - CROSSHAIR_SCREEN_MARGIN;
        Some((
            (x.round() as i32).clamp(CROSSHAIR_SCREEN_MARGIN, max_x),
            (y.round() as i32).clamp(CROSSHAIR_SCREEN_MARGIN, max_y),
        ))
    }

    fn ship_visible(&self) -> bool {
        if self.program_mode != ProgramMode::Game {
            return true;
        }
        if self.ship_absorbed || self.ending_cause == Some("NAVE DESTRUIDA") {
            return false;
        }
        self.invulnerable_timer <= 0.0 || (self.time * BLINK_RATE).sin() > 0.0
    }

    fn aimed_target(&self) -> Option<(String, f32)> {
        if self.program_mode != ProgramMode::Game {
            return None;
        }
        let origin = self.ship.position;
        let forward = self.ship.forward();
        let score = |position: Vec3| {
            let offset = position - origin;
            let distance = offset.length();
            let alignment = offset.dot(forward) / distance.max(f32::EPSILON);
            (distance <= TARGET_RANGE && alignment >= TARGET_CONE).then_some(distance)
        };
        let enemy = self
            .run_enemies
            .enemies()
            .iter()
            .filter_map(|enemy| {
                score(enemy.position)
                    .map(|distance| (distance, "ENEMIGO".to_owned(), enemy.health / enemy.max_health))
            });
        let planets = self.run_world.planets().filter_map(|planet| {
            score(planet.position).map(|distance| {
                (
                    distance,
                    format!("PLANETA {}", planet_name(planet.kind)),
                    planet.health / planet.max_health,
                )
            })
        });
        enemy
            .chain(planets)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, name, ratio)| (name, ratio.clamp(0.0, 1.0)))
    }

    fn open_roulette(&mut self) {
        let options = upgrade_options(&self.weapons.inventory, &mut self.reward_random, 3);
        let level = self.progression.level() + 1 - self.pending_upgrades.max(1);
        self.roulette = Some(Roulette::new(options, level));
        self.session.begin_upgrade_roulette();
    }

    fn update_upgrades(&mut self, input: &InputState, delta_time: f32) {
        if self.program_mode != ProgramMode::Game {
            return;
        }
        if self.roulette.is_none()
            && self.pending_upgrades > 0
            && self.session.state() == RunState::Playing
        {
            self.open_roulette();
        }
        if self.session.state() != RunState::UpgradeRoulette {
            return;
        }
        let Some(roulette) = &mut self.roulette else {
            self.session.resume_playing();
            return;
        };
        if roulette.update(delta_time) {
            self.sounds.push(SoundEffect::RouletteTick);
        }
        if input.was_pressed(Key::A) || input.was_pressed(Key::Left) {
            roulette.move_selection(-1);
        }
        if input.was_pressed(Key::D) || input.was_pressed(Key::Right) {
            roulette.move_selection(1);
        }
        let mut choice = input
            .pressed_digit()
            .and_then(|index| roulette.choose(index));
        if choice.is_none() && input.was_pressed(Key::Enter) {
            choice = roulette.confirm();
        }
        if let Some(option) = choice {
            self.confirm_upgrade(option);
        }
    }

    fn confirm_upgrade(&mut self, option: upgrades::UpgradeOption) {
        apply_option(
            option,
            &mut self.weapons.inventory,
            &mut self.progression,
            &mut self.buffs,
        );
        self.pending_upgrades = self.pending_upgrades.saturating_sub(1);
        self.roulette = None;
        if self.pending_upgrades > 0 {
            self.session.resume_playing();
            self.open_roulette();
        } else {
            self.session.resume_playing();
            self.notify("MEJORA APLICADA");
        }
    }

    pub fn draw_overlays(&self, renderer: &mut Renderer) -> Vec<TextLabel> {
        match (&self.roulette, &self.summary) {
            (Some(roulette), _) if self.session.state() == RunState::UpgradeRoulette => {
                roulette.draw(renderer, &self.weapons.inventory)
            }
            (_, Some(summary)) if self.session.state() == RunState::Summary => {
                draw_summary(renderer, summary, SUMMARY_AUTO_RETURN - self.ending_timer)
            }
            _ => Vec::new(),
        }
    }

    fn aim(&self) -> Aim {
        Aim {
            origin: self.ship.position,
            forward: self.ship.forward(),
            right: self.ship.right(),
            up: self.ship.up(),
        }
    }

    fn weapon_modifiers(&self) -> Modifiers {
        self.buffs.modifiers()
    }

    pub fn take_sounds(&mut self) -> Vec<SoundEffect> {
        std::mem::take(&mut self.sounds)
    }

    pub fn run_stats(&self) -> RunStats {
        self.run_stats
    }

    pub fn weapon_levels(&self) -> Vec<(&'static str, u8)> {
        self.weapons
            .inventory
            .owned()
            .map(|(kind, level)| (kind.name(), level))
            .collect()
    }

    fn push_damage_number(&mut self, position: Vec3, amount: f32) {
        if let Some(existing) = self.damage_numbers.iter_mut().find(|number| {
            (number.position - position).length() < DAMAGE_NUMBER_MERGE
                && number.life > DAMAGE_NUMBER_LIFE * 0.5
        }) {
            existing.amount += amount;
            existing.life = DAMAGE_NUMBER_LIFE;
            return;
        }
        if self.damage_numbers.len() >= MAX_DAMAGE_NUMBERS {
            self.damage_numbers.remove(0);
        }
        self.damage_numbers.push(DamageNumber {
            position,
            amount,
            life: DAMAGE_NUMBER_LIFE,
        });
    }

    fn update_damage_numbers(&mut self, delta_time: f32) {
        let rise = self.ship.up() * (DAMAGE_NUMBER_RISE * delta_time);
        self.damage_numbers.retain_mut(|number| {
            number.life -= delta_time;
            number.position = number.position + rise;
            number.life > 0.0
        });
    }

    pub fn damage_labels(&self, camera: &Camera) -> Vec<TextLabel> {
        let view_projection = camera.view_projection_matrix();
        self.damage_numbers
            .iter()
            .filter_map(|number| {
                let clip = view_projection * number.position.extend(1.0);
                if clip.w <= 0.0 {
                    return None;
                }
                let x = (clip.x / clip.w + 1.0) * 0.5 * FRAMEBUFFER_WIDTH as f32;
                let y = (1.0 - clip.y / clip.w) * 0.5 * FRAMEBUFFER_HEIGHT as f32;
                Some(
                    TextLabel::new(
                        format!("{:.0}", number.amount),
                        x,
                        y,
                        DAMAGE_NUMBER_SIZE,
                        DAMAGE_NUMBER_COLOR,
                    )
                    .centered(),
                )
            })
            .collect()
    }

    fn draw_enemy_bars(&self, renderer: &mut Renderer) {
        let up = self.ship.up();
        let right = self.ship.right();
        for enemy in self.run_enemies.enemies() {
            let color = match enemy.kind {
                EnemyKind::Common => continue,
                EnemyKind::Resistant => RESISTANT_BAR_COLOR,
                EnemyKind::Elite => ELITE_BAR_COLOR,
            };
            if !enemy.is_materialized() {
                continue;
            }
            let half = enemy.radius() * 1.1;
            let center = enemy.position + up * (enemy.radius() * 1.5 + 0.4);
            let start = center - right * half;
            let end = center + right * half;
            renderer.draw_line_3d(start, end, ENEMY_BAR_BACKGROUND, false);
            let filled = start + (end - start) * enemy.health_ratio();
            renderer.draw_line_3d(start, filled, color, false);
            if enemy.kind == EnemyKind::Elite {
                let lift = up * 0.12;
                renderer.draw_line_3d(start + lift, filled + lift, color, false);
            }
        }
    }

    fn handle_beam_events(&mut self, events: enemy_beams::BeamEvents) {
        if events.fired > 0 {
            self.sounds.push(SoundEffect::Laser);
        }
        for impact in events.impacts.iter().take(MAX_BEAM_IMPACT_EFFECTS) {
            self.effects.impact(*impact);
        }
        for (position, kind) in &events.enemies_destroyed {
            self.enemy_explosion(*position, *kind);
        }
        if events.player_damage > 0.0 {
            self.ship.velocity = self.ship.velocity + events.player_knockback;
            self.damage_player(events.player_damage);
        }
    }

    fn enemy_explosion(&mut self, position: Vec3, kind: EnemyKind) {
        match kind {
            EnemyKind::Common => self.effects.enemy_explosion(position),
            EnemyKind::Resistant => self.effects.tough_enemy_explosion(position, false),
            EnemyKind::Elite => self.effects.tough_enemy_explosion(position, true),
        }
    }

    fn handle_combat_events(&mut self, events: CombatEvents) {
        for kind in &events.shots {
            self.sounds.push(match kind {
                weapons::WeaponKind::Laser => SoundEffect::Laser,
                weapons::WeaponKind::Bombs | weapons::WeaponKind::Boomerang => SoundEffect::Launch,
                weapons::WeaponKind::Aura => SoundEffect::Impact,
                _ => SoundEffect::Shot,
            });
        }
        if !events.impacts.is_empty() {
            self.sounds.push(SoundEffect::Impact);
        }
        if !events.enemies_destroyed.is_empty() {
            self.sounds.push(SoundEffect::EnemyDestroyed);
        }
        if !events.explosions.is_empty() {
            self.sounds.push(SoundEffect::BombExplosion);
        }
        if !events.planets_destroyed.is_empty() {
            self.sounds.push(SoundEffect::PlanetDestroyed);
        }
        for impact in events.impacts.iter().take(MAX_IMPACT_EFFECTS) {
            self.effects.impact(*impact);
        }
        for (position, impulse) in events.knockbacks.iter().take(MAX_IMPACT_EFFECTS) {
            self.effects.knockback(*position, *impulse);
        }
        for (position, amount) in &events.damage {
            self.push_damage_number(*position, *amount);
        }
        for (position, kind) in &events.enemies_destroyed {
            self.enemy_explosion(*position, *kind);
            self.run_stats.enemies_destroyed += 1;
            self.grant_player_experience(kind.experience());
        }
        for (center, radius) in &events.explosions {
            self.effects.bomb_explosion(*center, *radius);
        }
        for planet in &events.planets_destroyed {
            self.effects
                .planet_explosion(planet.position, planet.radius, planet.kind);
            self.run_stats.planets_destroyed += 1;
            self.reward_planet(*planet);
        }
    }

    fn reward_planet(&mut self, planet: DestroyedPlanet) {
        let reward = planet_reward(planet.kind, &self.weapons.inventory, &mut self.reward_random);
        let message = self.apply_reward(reward);
        self.notify(format!(
            "PLANETA {} DESTRUIDO - {}",
            planet_name(planet.kind),
            message
        ));
    }

    fn apply_reward(&mut self, reward: Reward) -> String {
        match reward {
            Reward::Experience(amount) => {
                self.grant_player_experience(amount);
                format!("+{} XP", amount)
            }
            Reward::Buff(kind) => {
                self.buffs.grant(kind);
                format!(
                    "{} X{:.2} {:.0}S",
                    kind.name(),
                    self.buffs.multiplier(kind),
                    self.buffs.remaining(kind)
                )
            }
            Reward::Improve(kind) => {
                self.weapons.inventory.upgrade(kind);
                format!("{} NV {}", kind.name(), self.weapons.inventory.level(kind))
            }
            Reward::Unlock(kind) => {
                self.weapons.inventory.unlock(kind);
                format!("NUEVA ARMA {}", kind.name())
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
        self.planet_collider_start = self.colliders.len();
        self.colliders.extend(self.run_world.colliders());
    }

    pub fn damage_player(&mut self, amount: f32) -> bool {
        if self.program_mode != ProgramMode::Game || self.session.state() != RunState::Playing
            || self.buffs.is_shielded()
            || self.ship.is_evasive_loop_invulnerable()
        {
            return false;
        }
        let destroyed = self.progression.take_damage(amount);
        self.damage_flash = DAMAGE_FLASH_DURATION;
        self.invulnerable_timer = INVULNERABILITY_DURATION;
        self.sounds.push(SoundEffect::Damage);
        if destroyed {
            self.session.destroy_player();
            self.effects.player_explosion(self.ship.position);
            self.ending_timer = 0.0;
            self.sounds.push(SoundEffect::PlanetDestroyed);
            self.ending_cause = Some("NAVE DESTRUIDA");
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
            self.pending_upgrades += gain.levels_gained;
            self.sounds.push(SoundEffect::LevelUp);
            self.notify(format!("NIVEL {}", self.progression.level()));
        }
        gain.levels_gained
    }

    fn update_sun_lifecycle(&mut self, delta_time: f32) {
        if self.program_mode != ProgramMode::Game {
            return;
        }
        if self.session.state() == RunState::Playing {
            let offset = follow_offset(self.system.sun_position(), self.ship.position, delta_time);
            self.system.translate_origin(offset);
        }
        let progress = self.collapse_progress();
        let shrink = (progress / 0.55).clamp(0.0, 1.0);
        let contraction = 1.0 - (1.0 - COLLAPSE_MIN_SUN) * shrink * shrink * (3.0 - 2.0 * shrink);
        self.system.config.bodies[0].radius =
            self.sun_base_radius * radius_multiplier(self.session.elapsed()) * contraction;
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
        if input.was_pressed(Key::G) {
            self.gravity_enabled = !self.gravity_enabled;
            self.notify(if self.gravity_enabled {
                "GRAVEDAD ACTIVADA"
            } else {
                "GRAVEDAD DESACTIVADA"
            });
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

    fn update_evasive_loop_input(&mut self, input: &InputState) {
        if !input.was_pressed(Key::R)
            || self.program_mode != ProgramMode::Game
            || self.session.state() != RunState::Playing
        {
            return;
        }
        if self.camera_mode != CameraMode::Follow || self.warp.is_traveling() {
            self.notify("ESQUIVA NO DISPONIBLE");
            return;
        }
        if self.ship.try_start_evasive_loop() {
            self.notify("LOOP EVASIVO");
        } else if self.ship.evasive_loop_cooldown() > 0.0 {
            self.notify(format!(
                "ESQUIVA RECARGANDO {:.1}S",
                self.ship.evasive_loop_cooldown()
            ));
        } else {
            self.notify("ESQUIVA NO DISPONIBLE");
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

    fn gravity_wells(&self) -> Vec<GravityWell> {
        (0..self.system.body_count())
            .map(|index| {
                GravityWell::new(self.system.position(index), self.system.body(index).radius)
            })
            .collect()
    }

    fn fly(&mut self, controls: &ShipControls, delta_time: f32) {
        match self.ship.mode {
            FlightMode::Free => {
                if self.gravity_enabled {
                    let pull = total_acceleration(self.ship.position, &self.gravity_wells());
                    self.ship.apply_acceleration(pull, delta_time);
                }
                self.ship.update_free(controls, delta_time);
            }
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
            None if self.ship.is_evasive_looping() => (
                self.ship.evasive_camera_forward(),
                self.ship.evasive_camera_up(),
                self.follow_distance,
                self.follow_distance * FOLLOW_HEIGHT_RATIO,
                FOLLOW_LOOK_AHEAD,
                EVASIVE_LOOP_CAMERA_SMOOTHING,
            ),
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

        for planet in self.run_world.planets() {
            let mut material = self.materials.run_planets[planet.kind.index()];
            material.base_color = Vec3::new(
                (material.base_color.x * planet.tint.x).clamp(0.0, 1.0),
                (material.base_color.y * planet.tint.y).clamp(0.0, 1.0),
                (material.base_color.z * planet.tint.z).clamp(0.0, 1.0),
            );
            if planet.kind == PlanetKind::Core {
                let pulse = 0.6 + 0.4 * (self.time * 2.5 + planet.rotation_phase).sin();
                material.emissive = material.emissive + CORE_GLOW * pulse;
            }
            scene.push(Entity {
                model: self.models.run_planets[planet.kind.index()],
                material,
                transform: Transform::new(planet.position)
                    .with_rotation(Mat4::rotation_y(
                        planet.rotation_phase + self.time * planet.spin_speed,
                    ))
                    .with_scale(Vec3::new(
                        planet.radius * planet.shape.x,
                        planet.radius * planet.shape.y,
                        planet.radius * planet.shape.z,
                    )),
                radius: planet.radius,
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
            let forward = enemy.forward();
            let mut material = self.materials.enemy;
            match enemy.kind {
                EnemyKind::Common => {}
                EnemyKind::Resistant => material.base_color = RESISTANT_COLOR,
                EnemyKind::Elite => {
                    material.base_color = ELITE_COLOR;
                    material.emissive = ELITE_GLOW;
                }
            }
            let size = ENEMY_RENDER_SCALE * enemy.kind.scale();
            material.opacity = enemy.arrival_progress();
            material.outline = material.outline && enemy.is_materialized();
            scene.push(Entity {
                model: Model::Mesh(self.models.enemy),
                material,
                transform: Transform::new(enemy.position)
                    .with_rotation(
                        basis_rotation(forward, Vec3::new(0.0, 1.0, 0.0))
                            * Mat4::rotation_y(MODEL_FORWARD_CORRECTION),
                    )
                    .with_uniform_scale(size * enemy.arrival_scale()),
                radius: size,
            });
        }

        if !self.ship_visible() {
            return;
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
        if self.program_mode == ProgramMode::Game {
            self.weapons.draw(renderer, self.aim(), self.time);
        }
        for planet in self.run_world.planets() {
            draw_planet_signature(renderer, planet, self.time);
        }
        self.effects.draw(renderer);
        if self.program_mode == ProgramMode::Game {
            self.enemy_beams.draw(renderer, self.time);
            self.draw_enemy_bars(renderer);
        }
        if self.program_mode == ProgramMode::Game && self.buffs.is_shielded() && self.ship_visible() {
            let pulse = 1.6 + (self.time * 5.0).sin() * 0.12;
            let center = self.ship.position;
            let axes = [
                (self.ship.forward(), self.ship.right()),
                (self.ship.up(), self.ship.right()),
                (self.ship.forward(), self.ship.up()),
            ];
            for (u_axis, v_axis) in axes {
                let points: Vec<Vec3> = (0..=24)
                    .map(|index| {
                        let angle = index as f32 / 24.0 * std::f32::consts::TAU;
                        center + (u_axis * angle.cos() + v_axis * angle.sin()) * pulse
                    })
                    .collect();
                for segment in points.windows(2).step_by(2) {
                    renderer.draw_line_3d(segment[0], segment[1], SHIELD_COLOR, false);
                }
            }
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
            weapons: if self.program_mode == ProgramMode::Game {
                self.weapons
                    .inventory
                    .owned()
                    .map(|(kind, level)| format!("{} NV{}", kind.name(), level))
                    .collect()
            } else {
                Vec::new()
            },
            target: self.aimed_target(),
            buffs: self
                .buffs
                .active()
                .map(|(kind, stacks, remaining)| {
                    format!("{} X{} {:.0}S", kind.name(), stacks, remaining.ceil())
                })
                .collect(),
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

fn flight_speed_multiplier(mode: ProgramMode, buff_multiplier: f32) -> f32 {
    if mode == ProgramMode::Game {
        GAME_FLIGHT_SPEED_MULTIPLIER * buff_multiplier
    } else {
        1.0
    }
}

fn draw_summary(renderer: &mut Renderer, summary: &RunSummary, remaining: f32) -> Vec<TextLabel> {
    const WIDTH: i32 = 260;
    const HEIGHT: i32 = 150;
    const TEXT: f32 = 6.0;
    const ROW: f32 = 11.0;
    let left = (FRAMEBUFFER_WIDTH as i32 - WIDTH) / 2;
    let top = (FRAMEBUFFER_HEIGHT as i32 - HEIGHT) / 2 - 10;
    let center = FRAMEBUFFER_WIDTH as f32 * 0.5;
    let frame = Color::new(150, 150, 160);
    renderer.draw_rect_2d(left, top, WIDTH, HEIGHT, Color::new(8, 4, 16));
    for (x0, y0, x1, y1) in [
        (left, top, left + WIDTH, top),
        (left, top + HEIGHT, left + WIDTH, top + HEIGHT),
        (left, top, left, top + HEIGHT),
        (left + WIDTH, top, left + WIDTH, top + HEIGHT),
    ] {
        renderer.draw_line_2d(x0, y0, x1, y1, frame);
    }
    let mut labels = vec![
        TextLabel::new("== RESULTADOS ==", center, top as f32 + 8.0, TEXT, Color::new(170, 92, 255))
            .centered(),
        TextLabel::new(summary.cause, center, top as f32 + 20.0, TEXT, Color::new(255, 64, 64))
            .centered(),
    ];
    let rows = [
        ("TIEMPO SOBREVIVIDO", hud::format_time(summary.survived)),
        ("NIVEL ALCANZADO", summary.level.to_string()),
        ("ENEMIGOS DESTRUIDOS", summary.enemies_destroyed.to_string()),
        ("PLANETAS DESTRUIDOS", summary.planets_destroyed.to_string()),
        ("SEMILLA", summary.seed.to_string()),
    ];
    let text = Color::new(214, 214, 214);
    for (index, (label, value)) in rows.iter().enumerate() {
        let y = top as f32 + 36.0 + index as f32 * ROW;
        labels.push(TextLabel::new(*label, left as f32 + 14.0, y, TEXT, text));
        labels.push(TextLabel::new(
            value.clone(),
            (left + WIDTH) as f32 - 70.0,
            y,
            TEXT,
            Color::new(255, 220, 120),
        ));
    }
    labels.push(
        TextLabel::new(
            format!("ARMAS: {}", summary.weapons.join("  ")),
            center,
            top as f32 + 36.0 + rows.len() as f32 * ROW + 4.0,
            TEXT,
            text,
        )
        .centered(),
    );
    labels.push(
        TextLabel::new(
            format!(
                "ENTER MENU ({:.0}S)   R REINTENTAR CON LA MISMA SEMILLA",
                remaining.max(0.0).ceil()
            ),
            center,
            (top + HEIGHT) as f32 - 12.0,
            TEXT,
            frame,
        )
        .centered(),
    );
    labels
}

fn planet_ring(center: Vec3, u_axis: Vec3, v_axis: Vec3, radius: f32, segments: usize) -> Vec<Vec3> {
    (0..=segments)
        .map(|index| {
            let angle = index as f32 / segments as f32 * std::f32::consts::TAU;
            center + (u_axis * angle.cos() + v_axis * angle.sin()) * radius
        })
        .collect()
}

fn draw_planet_signature(renderer: &mut Renderer, planet: &run_world::RunPlanet, time: f32) {
    let tilt = planet.rotation_phase;
    let normal = Vec3::new(tilt.sin() * 0.35, 1.0, tilt.cos() * 0.35).normalize();
    let u_axis = normal.cross(Vec3::new(1.0, 0.0, 0.0)).normalize();
    let v_axis = normal.cross(u_axis);
    let center = planet.position;
    match planet.kind {
        PlanetKind::Gas => {
            for (scale, color) in [(1.7, GAS_RING_COLOR), (2.15, GAS_RING_DIM)] {
                let points = planet_ring(center, u_axis, v_axis, planet.radius * scale, 64);
                for segment in points.windows(2).step_by(2) {
                    renderer.draw_line_3d(segment[0], segment[1], color, false);
                }
            }
        }
        PlanetKind::Droid => {
            for index in 0..DROID_SATELLITES {
                let angle = time * 0.8 + index as f32 * std::f32::consts::TAU / DROID_SATELLITES as f32;
                let orbit = planet.radius * 1.6;
                let position = center + (u_axis * angle.cos() + v_axis * angle.sin()) * orbit;
                let arm = planet.radius * 0.18;
                for axis in [u_axis, v_axis, normal] {
                    renderer.draw_line_3d(position - axis * arm, position + axis * arm, DROID_COLOR, false);
                }
            }
        }
        PlanetKind::Core => {
            let pulse = 1.25 + (time * 2.5 + tilt).sin() * 0.08;
            let points = planet_ring(center, u_axis, v_axis, planet.radius * pulse, 48);
            for segment in points.windows(2).skip(1).step_by(3) {
                renderer.draw_line_3d(segment[0], segment[1], CORE_HALO_COLOR, false);
            }
        }
        PlanetKind::Rocky => {
            let orbit = planet.radius * 1.35;
            for index in 0..ROCKY_DEBRIS {
                let angle = time * 0.3 + index as f32 * 0.9;
                let position = center + (u_axis * angle.cos() + v_axis * angle.sin()) * orbit;
                let nudge = normal * (planet.radius * 0.05);
                renderer.draw_line_3d(position - nudge, position + nudge, ROCKY_DEBRIS_COLOR, false);
            }
        }
    }
}

fn scaled_controls(controls: ShipControls, factor: f32) -> ShipControls {
    let factor = factor.clamp(0.0, 1.0);
    ShipControls {
        thrust: controls.thrust * factor,
        vertical: controls.vertical * factor,
        yaw: controls.yaw * factor,
        pitch: controls.pitch * factor,
        roll: controls.roll * factor,
        boost: controls.boost && factor > 0.5,
    }
}

pub fn planet_name(kind: PlanetKind) -> &'static str {
    match kind {
        PlanetKind::Rocky => "ROCOSO",
        PlanetKind::Gas => "GASEOSO",
        PlanetKind::Core => "NUCLEO",
        PlanetKind::Droid => "DROIDE",
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

    #[test]
    fn flight_speed_bonus_only_applies_in_game() {
        assert_eq!(flight_speed_multiplier(ProgramMode::Menu, 1.75), 1.0);
        assert_eq!(flight_speed_multiplier(ProgramMode::Game, 1.0), 1.5);
        assert_eq!(flight_speed_multiplier(ProgramMode::Game, 1.25), 1.875);
    }

    #[test]
    fn crosshair_projects_a_point_ahead_of_the_ship_without_world_state() {
        let (mut game, mut camera) = test_game();
        let menu_position = game.crosshair_screen_position(&camera).unwrap();
        game.configure_mode(ProgramMode::Game, &mut camera);

        let forward_position = game.crosshair_screen_position(&camera).unwrap();
        game.ship.set_orientation(camera.right(), camera.up());
        let turned_position = game.crosshair_screen_position(&camera).unwrap();

        assert!(menu_position.0 >= CROSSHAIR_SCREEN_MARGIN);
        assert!(menu_position.1 >= CROSSHAIR_SCREEN_MARGIN);
        assert_ne!(forward_position, turned_position);
        assert!(turned_position.0 >= CROSSHAIR_SCREEN_MARGIN);
        assert!(turned_position.0 < FRAMEBUFFER_WIDTH as i32);
        assert!(turned_position.1 >= CROSSHAIR_SCREEN_MARGIN);
        assert!(turned_position.1 < FRAMEBUFFER_HEIGHT as i32);
    }

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
    fn nearby_damage_numbers_merge_and_fade() {
        let (mut game, mut camera) = test_game();
        game.configure_mode(ProgramMode::Game, &mut camera);
        game.push_damage_number(Vec3::new(0.0, 0.0, -10.0), 10.0);
        game.push_damage_number(Vec3::new(0.5, 0.0, -10.0), 5.0);
        game.push_damage_number(Vec3::new(30.0, 0.0, -10.0), 7.0);
        assert_eq!(game.damage_numbers.len(), 2);
        assert_eq!(game.damage_numbers[0].amount, 15.0);
        game.update_damage_numbers(DAMAGE_NUMBER_LIFE + 0.1);
        assert!(game.damage_numbers.is_empty());
    }

    #[test]
    fn death_shows_results_and_enter_returns_to_menu() {
        let (mut game, mut camera) = test_game();
        game.configure_mode(ProgramMode::Game, &mut camera);
        assert!(game.damage_player(500.0));
        assert!(!game.ship_visible());
        let input = InputState::new();
        for _ in 0..200 {
            game.update_ending(&input, 1.0 / 60.0, &mut camera);
        }
        assert_eq!(game.session.state(), RunState::Summary);
        let summary = game.summary().unwrap();
        assert_eq!(summary.cause, "NAVE DESTRUIDA");
        assert_eq!(summary.weapons, vec!["RAYO NV1".to_owned()]);
        for _ in 0..(60 * 21) {
            game.update_ending(&input, 1.0 / 60.0, &mut camera);
        }
        assert_eq!(game.program_mode, ProgramMode::Menu);
    }

    #[test]
    fn solar_collapse_grows_a_black_hole_and_ends_the_run() {
        let (mut game, mut camera) = test_game();
        game.configure_mode(ProgramMode::Game, &mut camera);
        assert!(game.session.update(session::RUN_DURATION_SECONDS + 1.0));
        game.ending_cause = Some("EL SOL COLAPSO");
        let radius_before = game.system.body(0).radius;
        let input = InputState::new();
        for _ in 0..240 {
            game.update_ending(&input, 1.0 / 60.0, &mut camera);
            game.update_sun_lifecycle(1.0 / 60.0);
        }
        assert!(game.system.body(0).radius < radius_before);
        assert!(game.black_hole().1 > 0.5);
        for _ in 0..400 {
            game.update_ending(&input, 1.0 / 60.0, &mut camera);
        }
        assert_eq!(game.session.state(), RunState::Summary);
        assert_eq!(game.summary().unwrap().cause, "EL SOL COLAPSO");
    }

    #[test]
    fn level_ups_queue_roulettes_that_pause_the_run() {
        let (mut game, mut camera) = test_game();
        game.configure_mode(ProgramMode::Game, &mut camera);
        let levels = game.grant_player_experience(500);
        assert!(levels >= 2);
        let input = InputState::new();
        game.update_upgrades(&input, 1.0 / 60.0);
        assert_eq!(game.session.state(), RunState::UpgradeRoulette);
        assert!(!game.session.update(1.0));
        for _ in 0..levels {
            assert_eq!(game.session.state(), RunState::UpgradeRoulette);
            let roulette = game.roulette.as_mut().unwrap();
            roulette.update(5.0);
            let option = roulette.confirm().unwrap();
            game.confirm_upgrade(option);
        }
        assert_eq!(game.session.state(), RunState::Playing);
        assert_eq!(game.pending_upgrades, 0);
        assert!(game.roulette.is_none());
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
    fn evasive_loop_invulnerability_blocks_player_damage() {
        let (mut game, mut camera) = test_game();
        game.configure_mode(ProgramMode::Game, &mut camera);
        game.ship.speed_multiplier = GAME_FLIGHT_SPEED_MULTIPLIER;
        assert!(game.ship.try_start_evasive_loop());

        let health = game.progression.health();
        assert!(!game.damage_player(50.0));
        assert_eq!(game.progression.health(), health);

        game.ship.update_free(&ShipControls::default(), 0.34);
        assert!(!game.damage_player(50.0));
        assert_eq!(game.progression.health(), health - 50.0);
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

    #[test]
    fn elite_beam_reaches_a_still_player_after_its_warning() {
        let (mut game, mut camera) = test_game();
        let input = InputState::new();
        let mut effect = WarpEffect::new();
        game.configure_mode(ProgramMode::Game, &mut camera);
        game.ship.speed_multiplier = 0.0;
        let elite_position = game.ship.position + game.ship.forward() * 40.0;
        game.run_enemies
            .spawn_kind_at(elite_position, game.progression.level(), EnemyKind::Elite);
        let mut warned = false;
        for _ in 0..(60 * 4) {
            game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);
            warned |= game.enemy_beams.active_count() > 0;
            game.ship.position = START_POSITION;
            game.ship.velocity = Vec3::default();
        }
        assert!(warned);
        assert!(game.progression.health() < game.progression.max_health());
    }
}
