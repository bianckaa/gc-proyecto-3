use super::lighting::PointLight;
use super::rasterizer::Fragment;
use super::Material;
use crate::engine::math::Vec3;

const FBM_OCTAVES: u32 = 4;
const SUN_MAX_DITHER_INTENSITY: f32 = 0.86;

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
    let direction = input.fragment.local_position.normalize();
    let time = input.time;

    // Capa 1: flujo de plasma deformado para evitar que el ruido parezca estatico.
    let flow = Vec3::new(time * 0.18, -time * 0.12, time * 0.07);
    let coarse = fbm(direction * 3.2 + flow);
    let warped = direction * 5.0 + flow + Vec3::new(coarse, coarse * 0.6, -coarse) * 2.4;
    let turbulence = fbm(warped);

    // Capa 2: granulos pequenos que hierven a una velocidad diferente.
    let granules = value_noise(direction * 34.0 + Vec3::new(-time * 0.9, time * 0.7, 0.0));

    // Capa 3: filamentos brillantes formados por crestas del campo turbulento.
    let filaments = (1.0 - ((turbulence - 0.52).abs() * 7.5)).clamp(0.0, 1.0);

    // Capa 4: erupciones locales con ciclos independientes sobre la superficie.
    let eruption = solar_eruption(direction, time, Vec3::new(0.76, 0.37, 0.53), 0.00)
        .max(solar_eruption(
            direction,
            time,
            Vec3::new(-0.48, 0.72, 0.50),
            0.37,
        ))
        .max(solar_eruption(
            direction,
            time,
            Vec3::new(0.20, -0.64, 0.74),
            0.71,
        ));

    let deep_orange = input.material.base_color * 0.86;
    let hot_yellow = Vec3::new(1.0, 0.90, 0.32);
    let white_hot = Vec3::new(1.0, 1.0, 0.92);
    let plasma = mix(deep_orange, hot_yellow, (turbulence * 0.9 + granules * 0.2).min(1.0));
    let color = mix(mix(plasma, white_hot, filaments * 0.55), white_hot, eruption);
    let pulse = 0.92 + 0.08 * (time * 1.7).sin();
    let intensity =
        ((0.78 + turbulence * 0.48 + filaments * 0.38 + eruption * 0.70) * pulse)
            .min(SUN_MAX_DITHER_INTENSITY);
    Surface {
        color,
        intensity,
    }
}

fn solar_eruption(direction: Vec3, time: f32, center: Vec3, phase: f32) -> f32 {
    let center = center.normalize();
    let proximity = ((direction.dot(center) - 0.91) / 0.09).clamp(0.0, 1.0);
    let cycle = (time * 0.16 + phase).fract();
    let envelope = if cycle < 0.24 {
        (cycle / 0.24 * std::f32::consts::PI).sin()
    } else {
        0.0
    };
    proximity * proximity * envelope
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

fn shade_lava(input: &ShadeInput) -> Surface {
    let flow = Vec3::new(0.0, input.time * 0.3, input.time * 0.1);
    let field = fbm(input.fragment.local_position * 4.0 + flow);
    let crack = 1.0 - ((field - 0.5).abs() * 12.0).min(1.0);
    let crust = lit_intensity(input) * 0.3;
    Surface {
        color: mix(input.material.base_color, Vec3::new(1.0, 0.75, 0.2), crack),
        intensity: crust + crack,
    }
}

fn shade_rocky(input: &ShadeInput) -> Surface {
    let point = input.fragment.local_position;
    let boulders = fbm(point * 5.0);
    let grit = value_noise(point * 28.0);
    let roughness = 0.45 + boulders * 0.7 + (grit - 0.5) * 0.35;
    Surface {
        color: input.material.base_color * (0.8 + boulders * 0.3),
        intensity: lit_intensity(input) * roughness,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderKind {
    Lambert,
    Rocky,
    Lava,
    Toon,
    GasGiant,
    Sun,
}

pub fn shade(input: &ShadeInput) -> Surface {
    let surface = match input.material.shader {
        ShaderKind::Lambert => shade_lambert(input),
        ShaderKind::Rocky => shade_rocky(input),
        ShaderKind::Lava => shade_lava(input),
        ShaderKind::Toon => shade_toon(input),
        ShaderKind::GasGiant => shade_gas_giant(input),
        ShaderKind::Sun => shade_sun(input),
    };
    Surface {
        color: surface.color + input.material.emissive,
        ..surface
    }
}
