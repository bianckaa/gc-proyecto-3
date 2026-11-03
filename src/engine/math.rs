use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub rows: [[f32; 4]; 4],
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Self {
        let length = self.length();
        if length > 0.0 {
            self * (1.0 / length)
        } else {
            self
        }
    }
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Self {
        let length = self.length();
        if length > 0.0 {
            self * (1.0 / length)
        } else {
            self
        }
    }

    pub fn extend(self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }
}

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Self {
        let length = self.length();
        if length > 0.0 {
            self * (1.0 / length)
        } else {
            self
        }
    }

    pub fn truncate(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl Mat4 {
    pub const fn identity() -> Self {
        Self {
            rows: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl Mat4 {
    pub fn transform_normal(self, normal: Vec3) -> Vec3 {
        let row = |index: usize| {
            let r = self.rows[index];
            Vec3::new(r[0], r[1], r[2])
        };
        let (x_axis, y_axis, z_axis) = (row(0), row(1), row(2));
        Vec3::new(
            y_axis.cross(z_axis).dot(normal),
            z_axis.cross(x_axis).dot(normal),
            x_axis.cross(y_axis).dot(normal),
        )
        .normalize()
    }

    pub fn translation(offset: Vec3) -> Self {
        let mut result = Self::identity();
        result.rows[0][3] = offset.x;
        result.rows[1][3] = offset.y;
        result.rows[2][3] = offset.z;
        result
    }

    pub fn scale(factors: Vec3) -> Self {
        let mut result = Self::identity();
        result.rows[0][0] = factors.x;
        result.rows[1][1] = factors.y;
        result.rows[2][2] = factors.z;
        result
    }

    pub fn rotation_x(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        let mut result = Self::identity();
        result.rows[1][1] = cos;
        result.rows[1][2] = -sin;
        result.rows[2][1] = sin;
        result.rows[2][2] = cos;
        result
    }

    pub fn rotation_y(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        let mut result = Self::identity();
        result.rows[0][0] = cos;
        result.rows[0][2] = sin;
        result.rows[2][0] = -sin;
        result.rows[2][2] = cos;
        result
    }

    pub fn rotation_z(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        let mut result = Self::identity();
        result.rows[0][0] = cos;
        result.rows[0][1] = -sin;
        result.rows[1][0] = sin;
        result.rows[1][1] = cos;
        result
    }

    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let forward = (target - eye).normalize();
        let right = forward.cross(up).normalize();
        let true_up = right.cross(forward);
        Self {
            rows: [
                [right.x, right.y, right.z, -right.dot(eye)],
                [true_up.x, true_up.y, true_up.z, -true_up.dot(eye)],
                [-forward.x, -forward.y, -forward.z, forward.dot(eye)],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let focal = 1.0 / (fov_y * 0.5).tan();
        let depth_range = near - far;
        Self {
            rows: [
                [focal / aspect, 0.0, 0.0, 0.0],
                [0.0, focal, 0.0, 0.0],
                [
                    0.0,
                    0.0,
                    (far + near) / depth_range,
                    2.0 * far * near / depth_range,
                ],
                [0.0, 0.0, -1.0, 0.0],
            ],
        }
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

macro_rules! impl_vector_ops {
    ($name:ident, $($field:ident),+) => {
        impl Add for $name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self { $($field: self.$field + other.$field),+ }
            }
        }

        impl Sub for $name {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self { $($field: self.$field - other.$field),+ }
            }
        }

        impl Mul<f32> for $name {
            type Output = Self;
            fn mul(self, scalar: f32) -> Self {
                Self { $($field: self.$field * scalar),+ }
            }
        }

        impl Neg for $name {
            type Output = Self;
            fn neg(self) -> Self {
                Self { $($field: -self.$field),+ }
            }
        }
    };
}

impl_vector_ops!(Vec2, x, y);
impl_vector_ops!(Vec3, x, y, z);
impl_vector_ops!(Vec4, x, y, z, w);

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut rows = [[0.0; 4]; 4];
        for (row, result_row) in rows.iter_mut().enumerate() {
            for (column, cell) in result_row.iter_mut().enumerate() {
                *cell = (0..4)
                    .map(|k| self.rows[row][k] * other.rows[k][column])
                    .sum();
            }
        }
        Self { rows }
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, v: Vec4) -> Vec4 {
        let row = |r: [f32; 4]| r[0] * v.x + r[1] * v.y + r[2] * v.z + r[3] * v.w;
        Vec4::new(
            row(self.rows[0]),
            row(self.rows[1]),
            row(self.rows[2]),
            row(self.rows[3]),
        )
    }
}
