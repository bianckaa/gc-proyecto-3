use crate::engine::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub position: Vec3,
    pub ambient: f32,
}

impl PointLight {
    pub const fn new(position: Vec3, ambient: f32) -> Self {
        Self { position, ambient }
    }

    pub fn diffuse(&self, world_position: Vec3, normal: Vec3) -> f32 {
        let to_light = (self.position - world_position).normalize();
        normal.dot(to_light).max(0.0)
    }

    pub fn lambert(&self, world_position: Vec3, normal: Vec3) -> f32 {
        self.ambient + (1.0 - self.ambient) * self.diffuse(world_position, normal)
    }
}

const SPECULAR_POWER: f32 = 28.0;
const SPECULAR_STRENGTH: f32 = 0.22;
const SPECULAR_TINT: Vec3 = Vec3::new(1.0, 0.95, 0.85);
const ATMOSPHERE_POWER: f32 = 3.0;
const ATMOSPHERE_TINT: Vec3 = Vec3::new(0.32, 0.48, 0.78);
const ATMOSPHERE_NIGHT_SIDE: f32 = 0.2;
const EXPOSURE: f32 = 1.35;

pub fn realistic_color(
    color: Vec3,
    intensity: f32,
    normal: Vec3,
    world_position: Vec3,
    light: &PointLight,
    eye: Vec3,
) -> Vec3 {
    let to_light = (light.position - world_position).normalize();
    let view = (eye - world_position).normalize();
    let facing_light = normal.dot(to_light).max(0.0);
    let half = (to_light + view).normalize();
    let specular = normal.dot(half).max(0.0).powf(SPECULAR_POWER) * SPECULAR_STRENGTH * facing_light;
    let rim = (1.0 - normal.dot(view).max(0.0)).powf(ATMOSPHERE_POWER);
    let atmosphere = rim * (ATMOSPHERE_NIGHT_SIDE + (1.0 - ATMOSPHERE_NIGHT_SIDE) * facing_light);
    color * (intensity * EXPOSURE) + SPECULAR_TINT * specular + ATMOSPHERE_TINT * atmosphere
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realistic_shading_adds_a_rim_and_a_highlight() {
        let light = PointLight::new(Vec3::new(0.0, 0.0, 10.0), 0.1);
        let eye = Vec3::new(0.0, 0.0, 10.0);
        let base = Vec3::new(0.5, 0.4, 0.3);
        let front = realistic_color(base, 1.0, Vec3::new(0.0, 0.0, 1.0), Vec3::default(), &light, eye);
        let edge = realistic_color(base, 0.0, Vec3::new(1.0, 0.0, 0.0), Vec3::default(), &light, eye);
        assert!(front.x > base.x);
        assert!(edge.z > 0.0 && edge.z > edge.x);
    }

    #[test]
    fn facing_side_is_bright_and_far_side_keeps_ambient() {
        let light = PointLight::new(Vec3::new(0.0, 10.0, 0.0), 0.2);
        let up = Vec3::new(0.0, 1.0, 0.0);
        assert!((light.lambert(Vec3::default(), up) - 1.0).abs() < 0.001);
        assert!((light.lambert(Vec3::default(), -up) - 0.2).abs() < 0.001);
    }
}
