use super::math::{Mat4, Vec3};

pub const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Mat4,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::new(Vec3::default())
    }
}

impl Transform {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: Mat4::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn with_rotation(mut self, rotation: Mat4) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_uniform_scale(self, scale: f32) -> Self {
        self.with_scale(Vec3::new(scale, scale, scale))
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::translation(self.position) * self.rotation * Mat4::scale(self.scale)
    }

    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        (self.model_matrix() * point.extend(1.0)).truncate()
    }

    pub fn max_scale(&self) -> f32 {
        self.scale
            .x
            .abs()
            .max(self.scale.y.abs())
            .max(self.scale.z.abs())
    }
}

pub fn basis_rotation(forward: Vec3, up: Vec3) -> Mat4 {
    let forward = forward.normalize();
    let fallback_up = if forward.dot(up.normalize()).abs() > 0.999 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        up
    };
    let right = forward.cross(fallback_up).normalize();
    let up = right.cross(forward);
    let back = -forward;
    Mat4 {
        rows: [
            [right.x, up.x, back.x, 0.0],
            [right.y, up.y, back.y, 0.0],
            [right.z, up.z, back.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

pub fn rotate_vector(rotation: Mat4, vector: Vec3) -> Vec3 {
    (rotation * vector.extend(0.0)).truncate()
}

pub fn rotate_around(vector: Vec3, axis: Vec3, angle: f32) -> Vec3 {
    let axis = axis.normalize();
    let (sin, cos) = angle.sin_cos();
    vector * cos + axis.cross(vector) * sin + axis * (axis.dot(vector) * (1.0 - cos))
}

pub fn lerp(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    from + (to - from) * t
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn exponential_blend(response: f32, delta_time: f32) -> f32 {
    1.0 - (-response * delta_time).exp()
}
