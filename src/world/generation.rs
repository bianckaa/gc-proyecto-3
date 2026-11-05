use super::orbit::Orbit;
use crate::engine::math::{Mat4, Vec3};
use crate::engine::transform::{basis_rotation, rotate_around};
use crate::renderer::shaders::fbm;
use std::f32::consts::TAU;

const MAX_ATTEMPTS_PER_PROP: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeededRandom {
    state: u32,
}

impl SeededRandom {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9,
        }
    }

    pub fn next(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.state >> 8) as f32 / (1u32 << 24) as f32
    }

    pub fn range(&mut self, minimum: f32, maximum: f32) -> f32 {
        minimum + (maximum - minimum) * self.next()
    }

    pub fn unit_direction(&mut self) -> Vec3 {
        let height = self.range(-1.0, 1.0);
        let azimuth = self.next() * TAU;
        let ring = (1.0 - height * height).max(0.0).sqrt();
        Vec3::new(ring * azimuth.cos(), height, ring * azimuth.sin())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeltConfig {
    pub seed: u32,
    pub count: usize,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub thickness: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub orbit_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Asteroid {
    pub orbit: Orbit,
    pub height: f32,
    pub scale: Vec3,
    pub spin_speed: f32,
    pub spin_phase: f32,
}

impl Asteroid {
    pub fn position(&self, time: f32) -> Vec3 {
        self.orbit.offset(time) + Vec3::new(0.0, self.height, 0.0)
    }

    pub fn rotation(&self, time: f32) -> Mat4 {
        let angle = self.spin_phase + time * self.spin_speed;
        Mat4::rotation_y(angle) * Mat4::rotation_x(angle * 0.7)
    }

    pub fn radius(&self) -> f32 {
        self.scale.x.max(self.scale.y).max(self.scale.z)
    }
}

pub fn asteroid_belt(config: &BeltConfig) -> Vec<Asteroid> {
    let mut random = SeededRandom::new(config.seed);
    (0..config.count)
        .map(|_| {
            let radius = random.range(config.inner_radius, config.outer_radius);
            let size = config.min_size
                + (config.max_size - config.min_size) * random.next() * random.next();
            let speed = config.orbit_speed * (config.inner_radius / radius).sqrt();
            Asteroid {
                orbit: Orbit::circular(radius, speed, random.next() * TAU),
                height: random.range(-0.5, 0.5) * config.thickness,
                scale: Vec3::new(
                    size * random.range(0.8, 1.2),
                    size * random.range(0.55, 1.0),
                    size * random.range(0.8, 1.2),
                ),
                spin_speed: random.range(0.2, 0.8),
                spin_phase: random.next() * TAU,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropKind {
    Crystal,
    Boulder,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceConfig {
    pub seed: u32,
    pub count: usize,
    pub noise_frequency: f32,
    pub threshold: f32,
    pub min_scale: f32,
    pub max_scale: f32,
    pub crystal_ratio: f32,
    pub embed_depth: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceProp {
    pub direction: Vec3,
    pub heading: f32,
    pub scale: f32,
    pub kind: PropKind,
}

impl SurfaceProp {
    pub fn local_matrix(&self, surface_radius: f32) -> Mat4 {
        Mat4::translation(self.direction * surface_radius)
            * self.local_rotation()
            * Mat4::scale(Vec3::new(self.scale, self.scale, self.scale))
    }

    pub fn local_rotation(&self) -> Mat4 {
        let reference = if self.direction.y.abs() < 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let tangent = reference.cross(self.direction).normalize();
        let forward = rotate_around(tangent, self.direction, self.heading);
        basis_rotation(forward, self.direction)
    }
}

pub fn surface_density(config: &SurfaceConfig, direction: Vec3) -> f32 {
    let offset = Vec3::new(
        config.seed as f32 * 0.173,
        config.seed as f32 * 0.311,
        config.seed as f32 * 0.057,
    );
    fbm(direction * config.noise_frequency + offset)
}

pub fn scatter_surface(config: &SurfaceConfig) -> Vec<SurfaceProp> {
    let mut random = SeededRandom::new(config.seed);
    let mut props = Vec::with_capacity(config.count);
    let mut attempts = 0;
    while props.len() < config.count && attempts < config.count * MAX_ATTEMPTS_PER_PROP {
        attempts += 1;
        let direction = random.unit_direction();
        let heading = random.next() * TAU;
        let size = random.next();
        let kind_roll = random.next();
        if surface_density(config, direction) < config.threshold {
            continue;
        }
        props.push(SurfaceProp {
            direction,
            heading,
            scale: config.min_scale + (config.max_scale - config.min_scale) * size,
            kind: if kind_roll < config.crystal_ratio {
                PropKind::Crystal
            } else {
                PropKind::Boulder
            },
        });
    }
    props
}
