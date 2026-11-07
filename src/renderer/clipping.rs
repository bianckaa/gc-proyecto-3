use crate::engine::math::{Vec2, Vec3, Vec4};

const MAX_CLIPPED_VERTICES: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ClipVertex {
    pub position: Vec4,
    pub local_position: Vec3,
    pub world_position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub uv: Vec2,
}

impl ClipVertex {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position + (other.position - self.position) * t,
            local_position: self.local_position + (other.local_position - self.local_position) * t,
            world_position: self.world_position +(other.world_position - self.world_position) * t,
            normal: self.normal + (other.normal - self.normal) * t,
            tangent: self.tangent + (other.tangent - self.tangent) * t,
            uv: self.uv + (other.uv - self.uv) * t,
        }
    }

    fn near_distance(self) -> f32 {
        self.position.z + self.position.w
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ClippedPolygon {
    vertices: [ClipVertex; MAX_CLIPPED_VERTICES],
    count: usize,
}

impl ClippedPolygon {
    fn push(&mut self, vertex: ClipVertex) {
        self.vertices[self.count] = vertex;
        self.count += 1;
    }

    pub fn triangles(&self) -> impl Iterator<Item = [ClipVertex; 3]> + '_ {
        (2..self.count).map(|i| [self.vertices[0], self.vertices[i - 1], self.vertices[i]])
    }
}

pub fn clip_against_near(triangle: [ClipVertex; 3]) -> ClippedPolygon {
    let mut polygon = ClippedPolygon::default();
    for index in 0..3 {
        let current = triangle[index];
        let next = triangle[(index + 1) % 3];
        let current_distance = current.near_distance();
        let next_distance = next.near_distance();
        if current_distance >= 0.0 {
            polygon.push(current);
        }
        if (current_distance >= 0.0) != (next_distance >= 0.0) {
            let t = current_distance / (current_distance - next_distance);
            polygon.push(current.lerp(next, t));
        }
    }
    polygon
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(near_distance: f32) -> ClipVertex {
        ClipVertex {
            position: Vec4::new(0.0, 0.0, near_distance - 1.0, 1.0),
            ..ClipVertex::default()
        }
    }

    fn triangle_count(distances: [f32; 3]) -> usize {
        clip_against_near(distances.map(vertex)).triangles().count()
    }

    #[test]
    fn counts_triangles_for_each_case() {
        assert_eq!(triangle_count([1.0, 1.0, 1.0]), 1);
        assert_eq!(triangle_count([-1.0, -1.0, -1.0]), 0);
        assert_eq!(triangle_count([1.0, -1.0, -1.0]), 1);
        assert_eq!(triangle_count([1.0, 1.0, -1.0]), 2);
    }

    #[test]
    fn clipped_vertices_lie_on_or_in_front_of_near() {
        let polygon = clip_against_near([vertex(2.0), vertex(-2.0), vertex(1.0)]);
        for triangle in polygon.triangles() {
            for vertex in triangle {
                assert!(vertex.near_distance() >= -0.0001);
            }
        }
    }
}
