use super::rasterizer::ScreenVertex;
use crate::engine::math::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    pub fn from_view_projection(matrix: Mat4) -> Self {
        let row = |index: usize| {
            let r = matrix.rows[index];
            Vec4::new(r[0], r[1], r[2], r[3])
        };
        let (x, y, z, w) = (row(0), row(1), row(2), row(3));
        let planes = [w + x, w - x, w + y, w - y, w + z, w - z].map(|plane| {
            let length = plane.truncate().length();
            plane * (1.0 / length)
        });
        Self { planes }
    }

    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.truncate().dot(center) + plane.w >= -radius)
    }
}

pub fn is_back_facing(a: ScreenVertex, b: ScreenVertex, c: ScreenVertex) -> bool {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frustum_keeps_visible_spheres_and_rejects_the_rest() {
        let projection = Mat4::perspective(1.0, 1.5, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(projection);
        assert!(frustum.contains_sphere(Vec3::new(0.0, 0.0, -10.0), 1.0));
        assert!(!frustum.contains_sphere(Vec3::new(0.0, 0.0, 10.0), 1.0));
        assert!(!frustum.contains_sphere(Vec3::new(50.0, 0.0, -10.0), 1.0));
        assert!(!frustum.contains_sphere(Vec3::new(0.0, 0.0, -200.0), 1.0));
        assert!(frustum.contains_sphere(Vec3::new(0.0, 0.0, 1.0), 2.0));
    }

    #[test]
    fn counter_clockwise_on_screen_faces_the_camera() {
        let a = ScreenVertex::new(0.0, 10.0, 0.0);
        let b = ScreenVertex::new(10.0, 10.0, 0.0);
        let c = ScreenVertex::new(5.0, 0.0, 0.0);
        assert!(!is_back_facing(a, b, c));
        assert!(is_back_facing(a, c, b));
    }
}
