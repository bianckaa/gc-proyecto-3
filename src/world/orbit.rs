use crate::engine::math::{Mat4, Vec3};
use crate::engine::transform::rotate_vector;
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit {
    pub radius: f32,
    pub eccentricity: f32,
    pub speed: f32,
    pub phase: f32,
    pub inclination: f32,
    pub node: f32,
}

impl Orbit {
    pub const fn circular(radius: f32, speed: f32, phase: f32) -> Self {
        Self {
            radius,
            eccentricity: 0.0,
            speed,
            phase,
            inclination: 0.0,
            node: 0.0,
        }
    }

    pub const fn none() -> Self {
        Self::circular(0.0, 0.0, 0.0)
    }

    pub const fn with_tilt(mut self, inclination: f32, node: f32) -> Self {
        self.inclination = inclination;
        self.node = node;
        self
    }

    pub const fn with_eccentricity(mut self, eccentricity: f32) -> Self {
        self.eccentricity = eccentricity;
        self
    }

    pub fn angle(&self, time: f32) -> f32 {
        self.phase + self.speed * time
    }

    pub fn distance_at(&self, angle: f32) -> f32 {
        let eccentricity = self.eccentricity.clamp(0.0, 0.95);
        self.radius * (1.0 - eccentricity * eccentricity) / (1.0 + eccentricity * angle.cos())
    }

    pub fn offset_at_angle(&self, angle: f32) -> Vec3 {
        let distance = self.distance_at(angle);
        let flat = Vec3::new(distance * angle.cos(), 0.0, distance * angle.sin());
        let plane = Mat4::rotation_y(self.node) * Mat4::rotation_x(self.inclination);
        rotate_vector(plane, flat)
    }

    pub fn offset(&self, time: f32) -> Vec3 {
        self.offset_at_angle(self.angle(time))
    }

    pub fn direction_of_travel(&self, time: f32) -> Vec3 {
        let angle = self.angle(time);
        let step = if self.speed < 0.0 { -0.01 } else { 0.01 };
        (self.offset_at_angle(angle + step) - self.offset_at_angle(angle)).normalize()
    }

    pub fn period(&self) -> f32 {
        if self.speed.abs() <= f32::EPSILON {
            f32::INFINITY
        } else {
            TAU / self.speed.abs()
        }
    }

    pub fn path(&self, center: Vec3, segments: usize) -> Vec<Vec3> {
        (0..=segments)
            .map(|index| center + self.offset_at_angle(index as f32 / segments as f32 * TAU))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spin {
    pub speed: f32,
    pub axial_tilt: f32,
}

impl Spin {
    pub const fn new(speed: f32, axial_tilt: f32) -> Self {
        Self { speed, axial_tilt }
    }

    pub fn rotation(&self, time: f32) -> Mat4 {
        Mat4::rotation_z(self.axial_tilt) * Mat4::rotation_y(self.speed * time)
    }
}
