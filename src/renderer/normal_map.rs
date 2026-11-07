use super::rasterizer::Fragment;
use crate::engine::math::{Vec2, Vec3};

const MAP_SIZE: usize = 128;
const BASE_PERIOD: i32 = 8;
const OCTAVES: u32 = 3;
const DEFAULT_TILING: f32 = 6.0;
const DEFAULT_STRENGTH: f32 = 0.15;
const MIN_TANGENT_LENGTH: f32 = 0.0001;

pub struct NormalMap {
    normals: Vec<Vec3>,
    pub tiling: f32,
}

fn lattice_value(x: i32, y: i32, period: i32, seed: u32) -> f32 {
    let (x, y) = (x.rem_euclid(period) as u32, y.rem_euclid(period) as u32);
    let mixed = (x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ seed)
        .wrapping_mul(1_274_126_177);
    ((mixed ^ (mixed >> 15)) & 0xFFFF) as f32 / 65_535.0
}

fn periodic_noise(u: f32, v: f32, period: i32, seed: u32) -> f32 {
    let (x, y) = (u * period as f32, v * period as f32);
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy) = (smooth(x - x.floor()), smooth(y - y.floor()));
    let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let row = |dy: i32| {
        mix(
            lattice_value(ix, iy + dy, period, seed),
            lattice_value(ix + 1, iy + dy, period, seed),
            fx,
        )
    };
    mix(row(0), row(1), fy)
}

fn height(u: f32, v: f32, seed: u32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut period = BASE_PERIOD;
    for _ in 0..OCTAVES {
        total += periodic_noise(u, v, period, seed) * amplitude;
        amplitude *= 0.5;
        period *= 2;
    }
    total
}

impl NormalMap {
    pub fn procedural(seed: u32) -> Self {
        Self::from_height(|u, v| height(u, v, seed), DEFAULT_STRENGTH)
    }

    pub fn from_height(height_at: impl Fn(f32, f32) -> f32, strength: f32) -> Self {
        let heights: Vec<f32> = (0..MAP_SIZE * MAP_SIZE)
            .map(|index| {
                let (x, y) = (index % MAP_SIZE, index / MAP_SIZE);
                height_at(x as f32 / MAP_SIZE as f32, y as f32 / MAP_SIZE as f32)
            })
            .collect();
        let at = |x: isize, y: isize| {
            let wrap = |value: isize| value.rem_euclid(MAP_SIZE as isize) as usize;
            heights[wrap(y) * MAP_SIZE + wrap(x)]
        };
        let normals = (0..MAP_SIZE * MAP_SIZE)
            .map(|index| {
                let (x, y) = ((index % MAP_SIZE) as isize, (index / MAP_SIZE) as isize);
                let scale = strength * MAP_SIZE as f32 * 0.5;
                let slope_u = (at(x + 1, y) - at(x - 1, y)) * scale;
                let slope_v = (at(x, y + 1) - at(x, y - 1)) * scale;
                Vec3::new(-slope_u, -slope_v, 1.0).normalize()
            })
            .collect();
        Self {
            normals,
            tiling: DEFAULT_TILING,
        }
    }

    pub fn sample(&self, uv: Vec2) -> Vec3 {
        let texel = |coordinate: f32| {
            ((coordinate * self.tiling).rem_euclid(1.0) * MAP_SIZE as f32) as usize % MAP_SIZE
        };
        self.normals[texel(uv.y) * MAP_SIZE + texel(uv.x)]
    }

    pub fn perturb(&self, fragment: &Fragment) -> Vec3 {
        let normal = fragment.normal;
        let tangent = fragment.tangent - normal * normal.dot(fragment.tangent);
        if tangent.length() < MIN_TANGENT_LENGTH {
            return normal;
        }
        let tangent = tangent.normalize();
        let bitangent = normal.cross(tangent);
        let mapped = self.sample(fragment.uv);
        (tangent * mapped.x + bitangent * mapped.y + normal * mapped.z).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fragment(tangent: Vec3, uv: Vec2) -> Fragment {
        Fragment {
            x: 0,
            y: 0,
            local_position: Vec3::default(),
            world_position: Vec3::default(),
            normal: Vec3::new(0.0, 1.0, 0.0),
            tangent,
            uv,
        }
    }

    #[test]
    fn flat_height_keeps_the_surface_normal() {
        let map = NormalMap::from_height(|_, _| 0.5, DEFAULT_STRENGTH);
        let bent = map.perturb(&fragment(Vec3::new(1.0, 0.0, 0.0), Vec2::new(0.3, 0.7)));
        assert!((bent - Vec3::new(0.0, 1.0, 0.0)).length() < 0.0001);
    }

    #[test]
    fn bumpy_map_tilts_normals_and_keeps_them_unit_length() {
        let map = NormalMap::procedural(7);
        let tilted = (0..64)
            .map(|step| {
                let uv = Vec2::new(step as f32 / 64.0, 0.37);
                map.perturb(&fragment(Vec3::new(1.0, 0.0, 0.0), uv))
            })
            .inspect(|normal| assert!((normal.length() - 1.0).abs() < 0.001))
            .filter(|normal| normal.y < 0.999)
            .count();
        assert!(tilted > 0);
    }

    #[test]
    fn missing_tangent_falls_back_to_the_surface_normal() {
        let map = NormalMap::procedural(7);
        let bent = map.perturb(&fragment(Vec3::default(), Vec2::new(0.5, 0.5)));
        assert_eq!(bent, Vec3::new(0.0, 1.0, 0.0));
    }
}
