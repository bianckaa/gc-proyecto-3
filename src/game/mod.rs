pub mod boss_planets;
pub mod buffs;
pub mod effects;
pub mod enemies;
pub mod enemy_beams;
pub mod hud;
pub mod player;
pub mod progression;
pub mod roulette;
pub mod run_world;
mod runtime;
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
use crate::platform::window::TextLabel;
use crate::renderer::culling::Frustum;
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use crate::renderer::shaders::ShaderKind;
use crate::renderer::warp_effect::WarpEffect;
use crate::renderer::{Deformation, Material, Mesh, Renderer};
use crate::world::generation::SeededRandom;
use crate::world::generation::{asteroid_belt, scatter_surface, Asteroid, PropKind, SurfaceProp};
use crate::world::orbit::Orbit;
use crate::world::solar_system::MAX_TIME_SCALE;
use crate::world::{SolarSystem, SystemConfig};
use boss_planets::{BossPlanets, ENGAGE_RANGE as BOSS_ENGAGE_RANGE};
use buffs::{ActiveBuffs, BuffKind};
use effects::Effects;
use enemies::{EnemyKind, EnemySwarm, PlayerMotion, ENEMY_RENDER_SCALE};
use enemy_beams::{BeamOwner, BeamRequest, EnemyBeams};
use hud::HudInfo;
use player::{FlightMode, Ship, ShipControls, SHIP_RADIUS};
use progression::PlayerProgression;
use roulette::Roulette;
use run_world::{DestroyedPlanet, PlanetKind, RunWorld};
use runtime::setup::{apply_run_seed, load_models, patrols, surfaces};
use session::{generate_seed, RunSession, RunState};
use std::f32::consts::PI;
use sun_lifecycle::{follow_offset, radius_multiplier};
use upgrades::{apply_option, planet_reward, upgrade_options, Reward};
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
const BOSS_HEAL_REWARD: f32 = 20.0;
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
pub const START_POSITION: Vec3 = run_world::RUN_START_POSITION;
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
    boss_planets: BossPlanets,
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
            boss_planets: BossPlanets::default(),
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
        self.boss_planets.clear();
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
            self.run_world
                .scale_boss_health(self.progression.level(), self.session.elapsed());
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
            let capacity = self
                .wave_director
                .settings(self.session.elapsed())
                .max_active;
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
            let active_planets: Vec<_> = self.run_world.planets().copied().collect();
            self.boss_planets.update(
                &active_planets,
                self.ship.position,
                self.progression.level(),
                self.session.elapsed(),
                delta_time,
                &self.colliders,
                &mut self.run_enemies,
                &mut self.enemy_beams,
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
        TextLabel::new(
            "== RESULTADOS ==",
            center,
            top as f32 + 8.0,
            TEXT,
            Color::new(170, 92, 255),
        )
        .centered(),
        TextLabel::new(
            summary.cause,
            center,
            top as f32 + 20.0,
            TEXT,
            Color::new(255, 64, 64),
        )
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
