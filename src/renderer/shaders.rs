use super::lighting::PointLight;
use super::rasterizer::Fragment;
use super::Material;
use crate::engine::math::{Vec2, Vec3};
use std::sync::OnceLock;

const FBM_OCTAVES: u32 = 4;
const SUN_MAX_DITHER_INTENSITY: f32 = 0.86;
const ROCKY_CRATER_COUNT: usize = 18;
const GOLDEN_ANGLE: f32 = 2.399_963_1;
static ROCKY_CRATERS: OnceLock<[(Vec3, f32); ROCKY_CRATER_COUNT]> = OnceLock::new();

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
    let plasma = mix(
        deep_orange,
        hot_yellow,
        (turbulence * 0.9 + granules * 0.2).min(1.0),
    );
    let color = mix(
        mix(plasma, white_hot, filaments * 0.55),
        white_hot,
        eruption,
    );
    let pulse = 0.92 + 0.08 * (time * 1.7).sin();
    let intensity = ((0.78 + turbulence * 0.48 + filaments * 0.38 + eruption * 0.70) * pulse)
        .min(SUN_MAX_DITHER_INTENSITY);
    Surface { color, intensity }
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
        color: mix(
            input.material.base_color * 0.55,
            input.material.base_color,
            band,
        ),
        intensity: lit_intensity(input) * (0.45 + 0.55 * band),
    }
}

fn gas_hurricane(
    direction: Vec3,
    center: Vec2,
    radius: Vec2,
    time: f32,
    phase: f32,
) -> (f32, f32, f32) {
    let dx = (direction.x - center.x) / radius.x;
    let dy = (direction.y - center.y) / radius.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let body = smoothstep(1.0, 0.18, distance);
    let angle = dy.atan2(dx);
    let spiral_wave = (angle * 4.0 - distance * 17.0 + time * 1.8 + phase).sin() * 0.5 + 0.5;
    let arms = spiral_wave.powi(2) * body * smoothstep(0.10, 0.30, distance);
    let eye = smoothstep(0.20, 0.055, distance);
    (body, arms, eye)
}

fn shade_gas_planet(input: &ShadeInput) -> Surface {
    let direction = input.fragment.local_position.normalize();
    let time = input.time;

    // Capa 1: bandas latitudinales anchas con deriva lenta.
    let broad_warp = fbm(direction * 3.2 + Vec3::new(time * 0.05, 0.0, -time * 0.03));
    let latitude = direction.y + (broad_warp - 0.5) * 0.22;
    let broad_bands = (latitude * 18.0 + time * 0.16).sin() * 0.5 + 0.5;

    // Capa 2: corrientes finas que se desplazan a diferente velocidad.
    let jet_noise = fbm(direction * 8.0 + Vec3::new(-time * 0.12, time * 0.02, 0.0));
    let narrow_bands = (latitude * 43.0 + jet_noise * 5.0 - time * 0.28).sin() * 0.5 + 0.5;

    // Capa 3: remolinos fibrosos dentro de las corrientes.
    let wisps = (1.0 - (jet_noise - 0.5).abs() * 5.5).clamp(0.0, 1.0);

    // Capa 4: dos huracanes con ojo y brazos espirales animados.
    let hurricane_a = gas_hurricane(
        direction,
        Vec2::new(0.52, -0.28),
        Vec2::new(0.36, 0.21),
        time,
        0.0,
    );
    let hurricane_b = gas_hurricane(
        direction,
        Vec2::new(-0.56, 0.42),
        Vec2::new(0.27, 0.16),
        -time * 0.82,
        2.1,
    );
    let storms = hurricane_a.0.max(hurricane_b.0);
    let spiral_arms = hurricane_a.1.max(hurricane_b.1);
    let storm_eyes = hurricane_a.2.max(hurricane_b.2);

    // Capa 5: mezcla cromatica entre atmosfera profunda, nubes y tormentas.
    let deep = input.material.base_color * 0.28;
    let middle = input.material.base_color;
    let pale_cloud = Vec3::new(0.66, 0.90, 0.96);
    let storm_color = Vec3::new(0.76, 0.24, 0.12);
    let spiral_color = Vec3::new(1.0, 0.86, 0.56);
    let mut color = mix(deep, middle, broad_bands);
    color = mix(color, pale_cloud, narrow_bands * 0.48 + wisps * 0.20);
    color = mix(color, storm_color, storms * 0.42);
    color = mix(color, spiral_color, spiral_arms * 0.90);
    color = mix(color, deep * 0.16, storm_eyes * 0.96);

    let light = lit_intensity(input);
    let intensity = ((light * (0.42 + broad_bands * 0.30 + narrow_bands * 0.14)
        + wisps * 0.08
        + storms * 0.06
        + spiral_arms * 0.16)
        * (1.0 - storm_eyes * 0.42))
        .clamp(0.18, 0.86);
    Surface { color, intensity }
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

fn smoothstep(edge_start: f32, edge_end: f32, value: f32) -> f32 {
    let t = ((value - edge_start) / (edge_end - edge_start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn interval_mask(value: f32, minimum: f32, maximum: f32, softness: f32) -> f32 {
    smoothstep(minimum - softness, minimum + softness, value)
        * (1.0 - smoothstep(maximum - softness, maximum + softness, value))
}

fn box_mask(point: Vec3, minimum: Vec3, maximum: Vec3, softness: f32) -> f32 {
    interval_mask(point.x, minimum.x, maximum.x, softness)
        * interval_mask(point.y, minimum.y, maximum.y, softness)
        * interval_mask(point.z, minimum.z, maximum.z, softness)
}

fn rocky_height(direction: Vec3) -> (f32, f32, f32) {
    let continental = value_noise(direction * 2.15 + Vec3::new(3.7, -1.4, 2.2));
    let warp = value_noise(direction * 3.1 + Vec3::new(9.0, -4.0, 2.0)) - 0.5;
    let mountain_field =
        value_noise(direction * 7.0 + Vec3::new(warp * 2.3, -warp * 1.6, warp * 1.9));
    let ridge = (1.0 - (mountain_field * 2.0 - 1.0).abs())
        .clamp(0.0, 1.0)
        .powi(3);
    let land = smoothstep(0.36, 0.68, continental);
    let mountains = ridge * land;
    let grit = hash(
        (direction.x * 48.0).floor() as i32 + 4,
        (direction.y * 48.0).floor() as i32 + 7,
        (direction.z * 48.0).floor() as i32 + 2,
    );
    let height = (continental * 0.42 + mountains * 0.58 + (grit - 0.5) * 0.08).clamp(0.0, 1.0);
    (height, mountains, grit)
}

fn crater_field(direction: Vec3, center: Vec3, radius: f32) -> (f32, f32, f32) {
    let offset = direction - center;
    let maximum_radius = radius * 1.61;
    let distance_squared = offset.dot(offset);
    if distance_squared > maximum_radius * maximum_radius {
        return (0.0, 0.0, 0.0);
    }
    let distance = distance_squared.sqrt();
    let bowl = (1.0 - distance / radius).clamp(0.0, 1.0);
    let rim = (1.0 - (distance - radius).abs() / (radius * 0.18)).clamp(0.0, 1.0);
    let ejecta_radius = radius * 1.48;
    let ejecta = (1.0 - (distance - ejecta_radius).abs() / (radius * 0.13)).clamp(0.0, 1.0);
    (bowl * bowl, rim * rim, ejecta * ejecta)
}

fn distributed_crater(index: usize) -> (Vec3, f32) {
    let fraction = (index as f32 + 0.5) / ROCKY_CRATER_COUNT as f32;
    let height = 1.0 - fraction * 2.0;
    let ring = (1.0 - height * height).sqrt();
    let azimuth = index as f32 * GOLDEN_ANGLE;
    let center = Vec3::new(ring * azimuth.cos(), height, ring * azimuth.sin());
    let variation = hash(index as i32, 31, 7);
    let radius = 0.095 + variation * 0.095 + if index.is_multiple_of(6) { 0.045 } else { 0.0 };
    (center, radius)
}

fn rocky_craters() -> &'static [(Vec3, f32); ROCKY_CRATER_COUNT] {
    ROCKY_CRATERS.get_or_init(|| std::array::from_fn(distributed_crater))
}

fn erosion_field(direction: Vec3, height: f32, mountains: f32) -> f32 {
    let flow_point =
        direction * 15.0 + Vec3::new(height * 2.4, height * 5.5 - mountains * 1.8, -height * 2.1);
    let flow = value_noise(flow_point);
    let channel = (1.0 - (flow - 0.5).abs() * 24.0).clamp(0.0, 1.0);
    channel * smoothstep(0.32, 0.76, height) * (0.35 + mountains * 0.65)
}

fn terrain_occlusion(height: f32, mountains: f32, crater_bowl: f32, erosion: f32) -> f32 {
    let valley = 1.0 - smoothstep(0.24, 0.56, height);
    let mountain_cavity =
        (1.0 - smoothstep(0.08, 0.48, mountains)) * smoothstep(0.34, 0.66, height);
    (valley * 0.38 + mountain_cavity * 0.16 + crater_bowl * 0.46 + erosion * 0.12).clamp(0.0, 1.0)
}

fn shade_rocky_planet(input: &ShadeInput) -> Surface {
    let direction = input.fragment.local_position.normalize();
    let (height, mountains, grit) = rocky_height(direction);

    // Capa 1: valles oscuros y tierras minerales de baja altura.
    let valley = Vec3::new(0.10, 0.045, 0.025);
    let lowland = input.material.base_color;
    let mut color = mix(valley, lowland, smoothstep(0.22, 0.52, height));

    // Capa 2: cordilleras rocosas formadas por crestas de FBM.
    let mountain_rock = Vec3::new(0.58, 0.42, 0.27);
    color = mix(color, mountain_rock, smoothstep(0.22, 0.72, mountains));

    // Capa 3: cumbres claras y granulado mineral de alta frecuencia.
    let peak = Vec3::new(0.84, 0.80, 0.70);
    let peak_mask = smoothstep(0.68, 0.90, height) * smoothstep(0.28, 0.72, mountains);
    color = mix(color, peak, peak_mask);
    color = color * (0.88 + grit * 0.22);

    // Capa 4: tres crateres con cuencas oscuras y bordes elevados.
    let (mut bowl, mut rim, mut ejecta) = (0.0_f32, 0.0_f32, 0.0_f32);
    for &(center, radius) in rocky_craters() {
        let (sample_bowl, sample_rim, sample_ejecta) = crater_field(direction, center, radius);
        bowl = bowl.max(sample_bowl);
        rim = rim.max(sample_rim);
        ejecta = ejecta.max(sample_ejecta);
    }
    color = mix(color, valley * 0.65, bowl * 0.90);
    color = mix(color, peak, rim * 0.72);
    color = mix(color, mountain_rock, ejecta * 0.38);

    // Capa 5: canales de erosion estrechos que atraviesan laderas y cordilleras.
    let erosion = erosion_field(direction, height, mountains);
    color = mix(color, valley * 0.52, erosion * 0.64);

    // Capa 6: oclusion procedural para hundir visualmente valles y crateres.
    let occlusion = terrain_occlusion(height, mountains, bowl, erosion);
    color = color * (1.0 - occlusion * 0.32);

    // Capa 7: relieve aproximado reutilizando montanas y detalle ya calculados.
    let relief_light =
        (0.66 + mountains * 0.38 + (grit - 0.5) * 0.22 - erosion * 0.10).clamp(0.42, 1.08);
    let light = lit_intensity(input);
    let intensity = ((light * (0.32 + relief_light * 0.62) + rim * 0.16 + ejecta * 0.05)
        * (1.0 - occlusion * 0.28)
        * (1.0 - erosion * 0.12))
        .clamp(0.12, 0.86);

    Surface { color, intensity }
}

fn core_projection(point: Vec3) -> Vec2 {
    let direction = point.normalize();
    let (ax, ay, az) = (direction.x.abs(), direction.y.abs(), direction.z.abs());
    if ax >= ay && ax >= az {
        Vec2::new(direction.z, direction.y)
    } else if ay >= az {
        Vec2::new(direction.x, direction.z)
    } else {
        Vec2::new(direction.x, direction.y)
    }
}

fn panel_sample(uv: Vec2, columns: f32, rows: f32, seed: i32) -> (f32, f32, Vec2) {
    let scaled = Vec2::new(uv.x * columns, uv.y * rows);
    let cell_x = scaled.x.floor() as i32;
    let cell_y = scaled.y.floor() as i32;
    let local = Vec2::new(scaled.x.rem_euclid(1.0), scaled.y.rem_euclid(1.0));
    let value = hash(cell_x, cell_y, seed);
    let edge = local.x.min(1.0 - local.x).min(local.y.min(1.0 - local.y));
    (value, edge, local)
}

fn shade_core_planet(input: &ShadeInput) -> Surface {
    let uv = core_projection(input.fragment.local_position);

    // Capa 1: grandes placas con alturas visuales distintas.
    let (macro_panel, macro_edge, _) = panel_sample(uv, 3.5, 2.5, 17);

    // Capa 2: bloques medianos superpuestos para romper la cuadricula principal.
    let offset_uv = Vec2::new(uv.x + 0.19, uv.y - 0.13);
    let (medium_panel, medium_edge, medium_local) = panel_sample(offset_uv, 9.0, 6.0, 43);

    // Capa 3: juntas oscuras y conductos finos similares a un circuito urbano.
    let macro_seam = (1.0 - macro_edge * 32.0).clamp(0.0, 1.0);
    let medium_seam = (1.0 - medium_edge * 46.0).clamp(0.0, 1.0);
    let horizontal_conduit = if medium_panel > 0.58 {
        (1.0 - (medium_local.y - 0.68).abs() * 55.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let vertical_conduit = if medium_panel < 0.28 {
        (1.0 - (medium_local.x - 0.34).abs() * 55.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let conduit = horizontal_conduit.max(vertical_conduit);

    // Capa 4: modulos pequenos y puntos de energia repartidos por la superficie.
    let (module_cell, _, module_local) = panel_sample(uv, 22.0, 16.0, 91);
    let module_shape = if module_cell > 0.78
        && (module_local.x - 0.5).abs() < 0.24
        && (module_local.y - 0.5).abs() < 0.20
    {
        1.0
    } else {
        0.0
    };
    let energy = (conduit * 0.75 + module_shape).min(1.0);

    // Capa 5: luz amarilla que nace de los canales hundidos del modelo.
    // Se usa el centro propio de la esfera, no el centro del modelo completo,
    // porque las estructuras externas desplazan el origen al normalizarlo.
    let core_sphere_center = Vec3::new(0.0, -0.145_505, 0.0);
    let body_radius = (input.fragment.local_position - core_sphere_center).length();
    let inset_floor = smoothstep(0.505, 0.522, body_radius);
    let inset_depth = 1.0 - smoothstep(0.535, 0.562, body_radius);
    let inset_glow = (inset_floor * inset_depth).clamp(0.0, 1.0);

    let relief = (macro_panel * 0.48 + medium_panel * 0.34 + module_shape * 0.18)
        * (1.0 - macro_seam * 0.65)
        * (1.0 - medium_seam * 0.45);
    let dark_metal = input.material.base_color * 0.16;
    let pale_metal = Vec3::new(0.72, 0.79, 0.82);
    let panel_color = mix(dark_metal, pale_metal, 0.18 + relief * 0.82);
    let seam_color = input.material.base_color * 0.08;
    let with_seams = mix(
        panel_color,
        seam_color,
        (macro_seam * 0.82 + medium_seam * 0.55).min(1.0),
    );
    let energy_color = Vec3::new(0.28, 0.96, 1.0);
    let inset_light = Vec3::new(1.0, 0.72, 0.08);
    let color = mix(
        mix(with_seams, energy_color, energy * 0.88),
        inset_light,
        inset_glow * 0.92,
    );
    let light = lit_intensity(input);
    let pulse = 0.88 + 0.12 * (input.time * 2.1).sin().abs();
    let intensity = (light * (0.30 + relief * 0.78) + energy * 0.38 + inset_glow * pulse * 0.52)
        .clamp(0.16, 0.88);

    Surface { color, intensity }
}

fn shade_droid_planet(input: &ShadeInput) -> Surface {
    let uv = core_projection(input.fragment.local_position);

    // Capa 1: placas blindadas grandes con distintos niveles visuales.
    let (armor, armor_edge, armor_local) = panel_sample(uv, 4.5, 3.5, 137);
    let (subpanel, subpanel_edge, subpanel_local) =
        panel_sample(Vec2::new(uv.x - 0.11, uv.y + 0.17), 12.0, 8.0, 211);
    let plate_height = armor * 0.58 + subpanel * 0.42;

    // Capa 2: zanjas profundas entre placas y paneles secundarios.
    let armor_trench = (1.0 - armor_edge * 34.0).clamp(0.0, 1.0);
    let subpanel_trench = (1.0 - subpanel_edge * 52.0).clamp(0.0, 1.0);
    let trench = armor_trench.max(subpanel_trench * 0.72);

    // Capa 3: franjas diagonales industriales en placas seleccionadas.
    let stripe_coordinate = (uv.x * 17.0 + uv.y * 11.0).rem_euclid(2.0);
    let stripe = if armor > 0.68 && stripe_coordinate < 0.62 {
        1.0
    } else {
        0.0
    };
    let stripe_border =
        if armor > 0.68 && ((stripe_coordinate - 0.62).abs() < 0.07 || stripe_coordinate > 1.92) {
            1.0
        } else {
            0.0
        };

    // Capa 4: modulos rectangulares y respiraderos mecanicos pequenos.
    let (module_cell, _, module_local) = panel_sample(uv, 24.0, 18.0, 307);
    let module = if module_cell > 0.73
        && (module_local.x - 0.5).abs() < 0.28
        && (module_local.y - 0.5).abs() < 0.17
    {
        1.0
    } else {
        0.0
    };
    let vent = if module_cell < 0.16
        && (module_local.x - 0.5).abs() < 0.09
        && (module_local.y - 0.5).abs() < 0.38
    {
        1.0
    } else {
        0.0
    };

    // Capa 5: conductos de energia roja con pulsacion independiente.
    let horizontal_wire = if subpanel > 0.54 {
        (1.0 - (subpanel_local.y - 0.52).abs() * 64.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let vertical_wire = if subpanel < 0.25 {
        (1.0 - (subpanel_local.x - 0.64).abs() * 64.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let pulse = 0.62 + 0.38 * (input.time * 3.4 + armor_local.y * 8.0).sin().abs();
    let energy = horizontal_wire.max(vertical_wire).max(module * 0.82) * pulse;

    let black_metal = Vec3::new(0.032, 0.036, 0.044);
    let armor_metal = input.material.base_color;
    let worn_metal = Vec3::new(0.66, 0.68, 0.70);
    let stripe_metal = Vec3::new(0.22, 0.24, 0.28);
    let energy_red = Vec3::new(1.0, 0.06, 0.025);
    let mut color = mix(black_metal, armor_metal, 0.24 + plate_height * 0.70);
    color = mix(color, worn_metal, subpanel * 0.22);
    color = mix(color, black_metal, trench * 0.92 + vent * 0.78);
    color = mix(color, stripe_metal, stripe * 0.72);
    color = mix(color, black_metal, stripe_border * 0.88);
    color = mix(color, energy_red, energy * 0.94);

    let light = lit_intensity(input);
    let intensity =
        (light * (0.31 + plate_height * 0.72) + stripe * 0.10 + energy * 0.44).clamp(0.18, 0.88);
    Surface { color, intensity }
}

fn shade_eta_player(input: &ShadeInput) -> Surface {
    let point = input.fragment.local_position;
    let uv = core_projection(point);
    let (panel, panel_edge, panel_local) = panel_sample(uv, 13.0, 9.0, 401);
    let seam = (1.0 - panel_edge * 48.0).clamp(0.0, 1.0);

    // Carroceria amarilla con placas doradas y pequenos paneles claros.
    let yellow = input.material.base_color;
    let golden_shadow = Vec3::new(0.34, 0.20, 0.025);
    let pale_metal = Vec3::new(0.82, 0.78, 0.58);
    let dark_seam = Vec3::new(0.055, 0.045, 0.030);
    let mut color = mix(golden_shadow, yellow, 0.58 + panel * 0.34);
    color = mix(color, dark_seam, seam * 0.72);

    let panel_detail =
        if panel > 0.72 && (panel_local.x - 0.5).abs() < 0.30 && (panel_local.y - 0.5).abs() < 0.12
        {
            1.0
        } else {
            0.0
        };
    color = mix(color, pale_metal, panel_detail * 0.50);

    // R2-D2 es un componente separado del OBJ; esta caja corresponde a sus
    // limites despues de normalizar el modelo.
    let r2 = box_mask(
        point,
        Vec3::new(0.175, 0.088, -0.062),
        Vec3::new(0.355, 0.276, 0.172),
        0.014,
    );
    let r2_blue = Vec3::new(0.06, 0.30, 0.92);
    let r2_highlight = smoothstep(0.18, 0.27, point.y) * r2;
    color = mix(color, r2_blue, r2 * 0.96);
    color = mix(color, Vec3::new(0.78, 0.86, 0.94), r2_highlight * 0.48);

    let light = lit_intensity(input);
    let intensity =
        (light * (0.42 + panel * 0.46) + panel_detail * 0.08 + r2 * 0.16).clamp(0.16, 0.88);
    Surface { color, intensity }
}

fn shade_enemy_ship(input: &ShadeInput) -> Surface {
    let uv = core_projection(input.fragment.local_position);
    let (armor, armor_edge, armor_local) = panel_sample(uv, 9.0, 7.0, 503);
    let (detail, detail_edge, detail_local) =
        panel_sample(Vec2::new(uv.x + 0.13, uv.y - 0.09), 21.0, 15.0, 557);
    let trench = (1.0 - armor_edge * 42.0)
        .max((1.0 - detail_edge * 58.0) * 0.55)
        .clamp(0.0, 1.0);

    let purple_shadow = Vec3::new(0.075, 0.025, 0.12);
    let purple_armor = input.material.base_color;
    let gray_metal = Vec3::new(0.70, 0.73, 0.78);
    let dark_metal = Vec3::new(0.035, 0.030, 0.045);
    let mut color = mix(purple_shadow, purple_armor, 0.46 + armor * 0.48);
    color = mix(color, dark_metal, trench * 0.82);

    let gray_panel: f32 =
        if armor > 0.63 && (armor_local.x - 0.5).abs() < 0.32 && (armor_local.y - 0.5).abs() < 0.20
        {
            1.0
        } else {
            0.0
        };
    let gray_module: f32 = if detail > 0.76
        && (detail_local.x - 0.5).abs() < 0.24
        && (detail_local.y - 0.5).abs() < 0.15
    {
        1.0
    } else {
        0.0
    };
    let gray_detail = gray_panel.max(gray_module);
    color = mix(color, gray_metal, gray_detail * 0.96);

    let light = lit_intensity(input);
    let intensity =
        (light * (0.36 + armor * 0.54) + gray_detail * 0.20 + detail * 0.04).clamp(0.14, 0.86);
    Surface { color, intensity }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderKind {
    CorePlanet,
    DroidPlanet,
    EnemyShip,
    EtaPlayer,
    GasPlanet,
    Lambert,
    Rocky,
    RockyPlanet,
    Lava,
    Toon,
    GasGiant,
    Sun,
}

pub fn shade(input: &ShadeInput) -> Surface {
    let surface = match input.material.shader {
        ShaderKind::CorePlanet => shade_core_planet(input),
        ShaderKind::DroidPlanet => shade_droid_planet(input),
        ShaderKind::EnemyShip => shade_enemy_ship(input),
        ShaderKind::EtaPlayer => shade_eta_player(input),
        ShaderKind::GasPlanet => shade_gas_planet(input),
        ShaderKind::Lambert => shade_lambert(input),
        ShaderKind::Rocky => shade_rocky(input),
        ShaderKind::RockyPlanet => shade_rocky_planet(input),
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
