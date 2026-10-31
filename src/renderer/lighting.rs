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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_side_is_bright_and_far_side_keeps_ambient() {
        let light = PointLight::new(Vec3::new(0.0, 10.0, 0.0), 0.2);
        let up = Vec3::new(0.0, 1.0, 0.0);
        assert!((light.lambert(Vec3::default(), up) - 1.0).abs() < 0.001);
        assert!((light.lambert(Vec3::default(), -up) - 0.2).abs() < 0.001);
    }
}
