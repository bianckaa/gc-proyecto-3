use super::lighting::PointLight;
use super::rasterizer::Fragment;
use super::Material;
use crate::engine::math::Vec3;

const FBM_OCTAVES: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    pub color: Vec3,
    pub intensity: f32,
}

pub struct ShadeInput<'a> {
    pub fragment: &'a Fragment,
    pub material: &'a Material,
    pub light: &'a PointLight,
    pub time: f32,
}

fn hash(x: i32, y: i32, z: i32) -> f32 {
    let seed = (x as u32)
        .wrapping_mul(73_856_093)
        ^ (y as u32).wrapping_mul(19_349_663)
        ^ (z as u32).wrapping_mul(83_492_791);
    let mixed = (seed ^ (seed >> 13)).wrapping_mul(1_274_126_177);
    ((mixed ^ (mixed >> 16)) & 0xFFFF) as f32 / 65_535.0
}

pub fn value_noise(point: Vec3) -> f32 {
    let (ix, iy, iz) = (point.x.floor(), point.y.floor(), point.z.floor());
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy, fz) = (smooth(point.x - ix), smooth(point.y - iy), smooth(point.z - iz));
    let (ix, iy, iz) = (ix as i32, iy as i32, iz as i32);
    let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let row = |dy: i32, dz: i32| {
        mix(hash(ix, iy + dy, iz + dz), hash(ix + 1, iy + dy, iz + dz), fx)
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

fn lit_intensity(input: &ShadeInput) -> f32 {
    input
        .light
        .lambert(input.fragment.world_position, input.fragment.normal)
}

fn shade_lambert(input: &ShadeInput) -> Surface {
    Surface {
        color: input.material.base_color,
        intensity: lit_intensity(input),
    }
}

fn mix(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    from + (to - from) * t
}

fn shade_sun(input: &ShadeInput) -> Surface {
    let time = input.time;
    let point = input.fragment.local_position * 3.0 + Vec3::new(time * 0.4, time * 0.3, 0.0);
    let turbulence = fbm(point + Vec3::new(fbm(point), 0.0, 0.0) * 2.0);
    let pulse = 0.9 + 0.1 * (time * 2.0).sin();
    Surface {
        color: mix(input.material.base_color, Vec3::new(1.0, 1.0, 0.85), turbulence),
        intensity: (0.25 + turbulence * 1.1) * pulse,
    }
}

fn shade_gas_giant(input: &ShadeInput) -> Surface {
    let direction = input.fragment.local_position.normalize();
    let drift = Vec3::new(input.time * 0.05, 0.0, 0.0);
    let distortion = fbm(direction * 4.0 + drift) - 0.5;
    let latitude = direction.y + distortion * 0.25;
    let band = (latitude * 16.0).sin() * 0.5 + 0.5;
    Surface {
        color: mix(input.material.base_color * 0.55, input.material.base_color, band),
        intensity: lit_intensity(input) * (0.45 + 0.55 * band),
    }
}

fn shade_toon(input: &ShadeInput) -> Surface {
    let light = lit_intensity(input);
    let intensity = match light {
        l if l > 0.75 => 1.0,
        l if l > 0.45 => 0.6,
        l if l > 0.2 => 0.3,
        _ => 0.08,
    };
    Surface {
        color: input.material.base_color,
        intensity,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderKind {
    Lambert,
    Toon,
    GasGiant,
    Sun,
}

pub fn shade(input: &ShadeInput) -> Surface {
    let surface = match input.material.shader {
        ShaderKind::Lambert => shade_lambert(input),
        ShaderKind::Toon => shade_toon(input),
        ShaderKind::GasGiant => shade_gas_giant(input),
        ShaderKind::Sun => shade_sun(input),
    };
    Surface {
        color: surface.color + input.material.emissive,
        ..surface
    }
}
