use super::math::{Mat4, Vec3};

const DEFAULT_FOV_Y_DEGREES: f32 = 60.0;
const DEFAULT_NEAR: f32 = 0.1;
const DEFAULT_FAR: f32 = 500.0;

fn rotate_around(vector: Vec3, axis: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    vector * cos + axis.cross(vector) * sin + axis * (axis.dot(vector) * (1.0 - cos))
}

fn lerp(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    from + (to - from) * t
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: Vec3,
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    forward: Vec3,
    up: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, aspect: f32) -> Self {
        Self {
            position,
            fov_y: DEFAULT_FOV_Y_DEGREES.to_radians(),
            aspect,
            near: DEFAULT_NEAR,
            far: DEFAULT_FAR,
            forward: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
        }
    }

    pub fn forward(&self) -> Vec3 {
        self.forward
    }

    pub fn up(&self) -> Vec3 {
        self.up
    }

    pub fn right(&self) -> Vec3 {
        self.forward.cross(self.up).normalize()
    }

    fn set_orientation(&mut self, forward: Vec3, up: Vec3) {
        self.forward = forward.normalize();
        let right = self.forward.cross(up).normalize();
        self.up = right.cross(self.forward);
    }

    pub fn move_local(&mut self, forward: f32, right: f32, up: f32) {
        self.position =
            self.position + self.forward * forward + self.right() * right + self.up * up;
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32, roll: f32) {
        let right = self.right();
        let forward = rotate_around(self.forward, self.up, yaw);
        let forward = rotate_around(forward, right, pitch);
        let up = rotate_around(self.up, right, pitch);
        let up = rotate_around(up, forward.normalize(), roll);
        self.set_orientation(forward, up);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn follow(
        &mut self,
        target: Vec3,
        target_forward: Vec3,
        target_up: Vec3,
        distance: f32,
        height: f32,
        look_ahead: f32,
        smoothing: f32,
        delta_time: f32,
    ) {
        let blend = 1.0 - (-smoothing * delta_time).exp();
        let desired_position = target - target_forward * distance + target_up * height;
        self.position = lerp(self.position, desired_position, blend);
        let desired_forward = (target + target_forward * look_ahead - self.position).normalize();
        self.set_orientation(
            lerp(self.forward, desired_forward, blend),
            lerp(self.up, target_up, blend),
        );
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.position + self.forward, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov_y, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follow_settles_behind_and_above_target() {
        let mut camera = Camera::new(Vec3::new(50.0, 0.0, 0.0), 1.0);
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        for _ in 0..600 {
            camera.follow(Vec3::default(), forward, up, 5.0, 2.0, 6.0, 6.0, 1.0 / 60.0);
        }
        assert!((camera.position - Vec3::new(0.0, 2.0, 5.0)).length() < 0.001);
    }

    #[test]
    fn follow_converges_equally_at_different_frame_rates() {
        let run = |steps: usize| {
            let mut camera = Camera::new(Vec3::new(10.0, 0.0, 0.0), 1.0);
            let forward = Vec3::new(0.0, 0.0, -1.0);
            let up = Vec3::new(0.0, 1.0, 0.0);
            for _ in 0..steps {
                camera.follow(Vec3::default(), forward, up, 5.0, 2.0, 6.0, 6.0, 0.5 / steps as f32);
            }
            camera.position
        };
        assert!((run(30) - run(120)).length() < 0.001);
    }
}
