use super::generation::{BeltConfig, SurfaceConfig};
use super::orbit::{Orbit, Spin};
use crate::engine::collision::Collider;
use crate::engine::math::{Mat4, Vec3};
use crate::renderer::shaders::ShaderKind;
use crate::renderer::Material;

pub const MAX_TIME_SCALE: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyModel {
    Planet,
    Core,
    Droid,
    Sphere,
}

impl BodyModel {
    pub fn file_name(self) -> Option<&'static str> {
        match self {
            BodyModel::Planet => Some("Planet.obj"),
            BodyModel::Core => Some("CORE_.obj"),
            BodyModel::Droid => Some("Droid_confed.obj"),
            BodyModel::Sphere => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyConfig {
    pub name: &'static str,
    pub model: BodyModel,
    pub material: Material,
    pub radius: f32,
    pub collision_scale: f32,
    pub flattening: f32,
    pub parent: Option<usize>,
    pub orbit: Orbit,
    pub spin: Spin,
    pub surface: Option<SurfaceConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemConfig {
    pub bodies: Vec<BodyConfig>,
    pub belt: BeltConfig,
    pub time_scale: f32,
    pub light_ambient: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyState {
    pub position: Vec3,
    pub rotation: Mat4,
}

#[derive(Debug, Clone)]
pub struct SolarSystem {
    pub config: SystemConfig,
    orbital_time: f32,
    states: Vec<BodyState>,
    previous_positions: Vec<Vec3>,
}

const fn material(base_color: Vec3, emissive: Vec3, shader: ShaderKind, outline: bool) -> Material {
    Material {
        base_color,
        emissive,
        shader,
        outline,
    }
}

impl SystemConfig {
    pub fn standard() -> Self {
        let none = Vec3::new(0.0, 0.0, 0.0);
        Self {
            bodies: vec![
                BodyConfig {
                    name: "SOL",
                    model: BodyModel::Planet,
                    material: material(
                        Vec3::new(1.0, 0.19, 0.015),
                        Vec3::new(0.08, 0.025, 0.0),
                        ShaderKind::Sun,
                        true,
                    ),
                    radius: 7.0,
                    collision_scale: 1.0,
                    flattening: 1.0,
                    parent: None,
                    orbit: Orbit::none(),
                    spin: Spin::new(0.05, -0.1),
                    surface: None,
                },
                BodyConfig {
                    name: "VULCANO",
                    model: BodyModel::Sphere,
                    material: material(
                        Vec3::new(1.0, 0.25, 0.1),
                        Vec3::new(0.05, 0.01, 0.0),
                        ShaderKind::Lava,
                        true,
                    ),
                    radius: 1.6,
                    collision_scale: 1.0,
                    flattening: 1.0,
                    parent: Some(0),
                    orbit: Orbit::circular(17.0, 0.09, 0.4).with_tilt(0.05, 0.3),
                    spin: Spin::new(0.3, 0.2),
                    surface: None,
                },
                BodyConfig {
                    name: "NUCLEO",
                    model: BodyModel::Core,
                    material: material(
                        Vec3::new(0.38, 0.48, 0.54),
                        Vec3::new(0.01, 0.025, 0.03),
                        ShaderKind::CorePlanet,
                        true,
                    ),
                    radius: 2.8,
                    collision_scale: 0.78,
                    flattening: 1.0,
                    parent: Some(0),
                    orbit: Orbit::circular(28.0, 0.06, 2.1)
                        .with_tilt(-0.06, 1.2)
                        .with_eccentricity(0.06),
                    spin: Spin::new(0.18, -0.16),
                    surface: None,
                },
                BodyConfig {
                    name: "ROCOSO",
                    model: BodyModel::Planet,
                    material: material(
                        Vec3::new(0.40, 0.18, 0.07),
                        none,
                        ShaderKind::RockyPlanet,
                        true,
                    ),
                    radius: 3.2,
                    collision_scale: 1.0,
                    flattening: 1.0,
                    parent: Some(0),
                    orbit: Orbit::circular(41.0, 0.045, 4.0).with_tilt(0.07, 2.4),
                    spin: Spin::new(0.08, -0.16),
                    surface: Some(SurfaceConfig {
                        seed: 7_301,
                        count: 70,
                        noise_frequency: 2.6,
                        threshold: 0.5,
                        min_scale: 0.14,
                        max_scale: 0.42,
                        crystal_ratio: 0.55,
                        embed_depth: 0.05,
                    }),
                },
                BodyConfig {
                    name: "GASEOSO",
                    model: BodyModel::Planet,
                    material: material(
                        Vec3::new(0.20, 0.46, 0.72),
                        Vec3::new(0.0, 0.015, 0.025),
                        ShaderKind::GasPlanet,
                        false,
                    ),
                    radius: 4.6,
                    collision_scale: 1.0,
                    flattening: 1.0,
                    parent: Some(0),
                    orbit: Orbit::circular(68.0, 0.03, 1.0).with_tilt(-0.03, 0.8),
                    spin: Spin::new(0.12, -0.16),
                    surface: None,
                },
                BodyConfig {
                    name: "DROIDE",
                    model: BodyModel::Droid,
                    material: material(
                        Vec3::new(0.46, 0.48, 0.52),
                        none,
                        ShaderKind::DroidPlanet,
                        true,
                    ),
                    radius: 6.0,
                    collision_scale: 0.95,
                    flattening: 0.36,
                    parent: Some(0),
                    orbit: Orbit::circular(90.0, 0.022, 3.2).with_tilt(0.02, 0.0),
                    spin: Spin::new(0.06, 0.0),
                    surface: None,
                },
                BodyConfig {
                    name: "LUNA",
                    model: BodyModel::Sphere,
                    material: material(Vec3::new(0.25, 0.75, 1.0), none, ShaderKind::Toon, true),
                    radius: 1.0,
                    collision_scale: 1.0,
                    flattening: 1.0,
                    parent: Some(4),
                    orbit: Orbit::circular(9.5, 0.32, 0.0).with_tilt(0.3, 0.0),
                    spin: Spin::new(0.32, 0.0),
                    surface: Some(SurfaceConfig {
                        seed: 515,
                        count: 18,
                        noise_frequency: 3.0,
                        threshold: 0.48,
                        min_scale: 0.08,
                        max_scale: 0.16,
                        crystal_ratio: 1.0,
                        embed_depth: 0.03,
                    }),
                },
            ],
            belt: BeltConfig {
                seed: 4_242,
                count: 130,
                inner_radius: 48.0,
                outer_radius: 54.0,
                thickness: 3.5,
                min_size: 0.25,
                max_size: 1.2,
                orbit_speed: 0.035,
            },
            time_scale: 1.0,
            light_ambient: 0.16,
        }
    }
}

impl SolarSystem {
    pub fn new(config: SystemConfig) -> Self {
        let count = config.bodies.len();
        let mut system = Self {
            config,
            orbital_time: 0.0,
            states: vec![
                BodyState {
                    position: Vec3::default(),
                    rotation: Mat4::identity(),
                };
                count
            ],
            previous_positions: vec![Vec3::default(); count],
        };
        system.recompute();
        system.previous_positions = system.states.iter().map(|state| state.position).collect();
        system
    }

    pub fn update(&mut self, delta_time: f32) {
        self.config.time_scale = self.config.time_scale.clamp(0.0, MAX_TIME_SCALE);
        self.previous_positions = self.states.iter().map(|state| state.position).collect();
        self.orbital_time += delta_time * self.config.time_scale;
        self.recompute();
    }

    fn recompute(&mut self) {
        if self.states.len() != self.config.bodies.len() {
            self.states.resize(
                self.config.bodies.len(),
                BodyState {
                    position: Vec3::default(),
                    rotation: Mat4::identity(),
                },
            );
            self.previous_positions
                .resize(self.config.bodies.len(), Vec3::default());
        }
        for index in 0..self.config.bodies.len() {
            let body = self.config.bodies[index];
            let center = self.parent_center(index);
            self.states[index] = BodyState {
                position: center + body.orbit.offset(self.orbital_time),
                rotation: body.spin.rotation(self.orbital_time),
            };
        }
    }

    fn parent_center(&self, index: usize) -> Vec3 {
        self.config.bodies[index]
            .parent
            .filter(|&parent| parent < index)
            .map_or(Vec3::default(), |parent| self.states[parent].position)
    }

    pub fn orbital_time(&self) -> f32 {
        self.orbital_time
    }

    pub fn body_count(&self) -> usize {
        self.config.bodies.len()
    }

    pub fn body(&self, index: usize) -> &BodyConfig {
        &self.config.bodies[index]
    }

    pub fn state(&self, index: usize) -> BodyState {
        self.states[index]
    }

    pub fn position(&self, index: usize) -> Vec3 {
        self.states[index].position
    }

    pub fn displacement(&self, index: usize) -> Vec3 {
        self.states[index].position - self.previous_positions[index]
    }

    pub fn sun_position(&self) -> Vec3 {
        self.states
            .first()
            .map_or(Vec3::default(), |state| state.position)
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.config.bodies.iter().position(|body| body.name == name)
    }

    pub fn model_matrix(&self, index: usize) -> Mat4 {
        let radius = self.config.bodies[index].radius;
        let state = self.states[index];
        Mat4::translation(state.position)
            * state.rotation
            * Mat4::scale(Vec3::new(radius, radius, radius))
    }

    pub fn surface_frame(&self, index: usize) -> Mat4 {
        let state = self.states[index];
        Mat4::translation(state.position) * state.rotation
    }

    pub fn collider(&self, index: usize) -> Collider {
        let body = &self.config.bodies[index];
        Collider::flattened(
            self.states[index].position,
            body.radius * body.collision_scale,
            body.flattening,
        )
    }

    pub fn colliders(&self) -> Vec<Collider> {
        (0..self.body_count())
            .map(|index| self.collider(index))
            .collect()
    }

    pub fn orbit_path(&self, index: usize, segments: usize) -> Option<Vec<Vec3>> {
        let body = &self.config.bodies[index];
        body.parent?;
        Some(body.orbit.path(self.parent_center(index), segments))
    }

    pub fn nearest_body(&self, point: Vec3) -> Option<(usize, f32)> {
        (0..self.body_count())
            .map(|index| {
                let surface_distance =
                    (self.position(index) - point).length() - self.config.bodies[index].radius;
                (index, surface_distance)
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}
