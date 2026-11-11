use super::*;

pub(super) fn hash(x: i32, y: i32, z: i32) -> f32 {
    let seed = (x as u32).wrapping_mul(73_856_093)
        ^ (y as u32).wrapping_mul(19_349_663)
        ^ (z as u32).wrapping_mul(83_492_791);
    let mixed = (seed ^ (seed >> 13)).wrapping_mul(1_274_126_177);
    ((mixed ^ (mixed >> 16)) & 0xFFFF) as f32 / 65_535.0
}

pub fn value_noise(point: Vec3) -> f32 {
    let (ix, iy, iz) = (point.x.floor(), point.y.floor(), point.z.floor());
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy, fz) = (
        smooth(point.x - ix),
        smooth(point.y - iy),
        smooth(point.z - iz),
    );
    let (ix, iy, iz) = (ix as i32, iy as i32, iz as i32);
    let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let row = |dy: i32, dz: i32| {
        mix(
            hash(ix, iy + dy, iz + dz),
            hash(ix + 1, iy + dy, iz + dz),
            fx,
        )
    };
    mix(
        mix(row(0, 0), row(1, 0), fy),
        mix(row(0, 1), row(1, 1), fy),
        fz,
    )
}

pub fn fbm(point: Vec3) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    let mut range = 0.0;
    for _ in 0..FBM_OCTAVES {
        total += value_noise(point * frequency) * amplitude;
        range += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    total / range
}

pub(super) fn lit_intensity(input: &ShadeInput) -> f32 {
    input
        .light
        .lambert(input.fragment.world_position, input.fragment.normal)
}

pub(super) fn shade_lambert(input: &ShadeInput) -> Surface {
    Surface {
        color: input.material.base_color,
        intensity: lit_intensity(input),
    }
}

pub(super) fn mix(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    from + (to - from) * t
}

pub(super) fn smoothstep(edge_start: f32, edge_end: f32, value: f32) -> f32 {
    let t = ((value - edge_start) / (edge_end - edge_start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn interval_mask(value: f32, minimum: f32, maximum: f32, softness: f32) -> f32 {
    smoothstep(minimum - softness, minimum + softness, value)
        * (1.0 - smoothstep(maximum - softness, maximum + softness, value))
}

pub(super) fn box_mask(point: Vec3, minimum: Vec3, maximum: Vec3, softness: f32) -> f32 {
    interval_mask(point.x, minimum.x, maximum.x, softness)
        * interval_mask(point.y, minimum.y, maximum.y, softness)
        * interval_mask(point.z, minimum.z, maximum.z, softness)
}
