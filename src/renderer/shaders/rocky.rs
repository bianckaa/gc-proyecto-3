use super::*;

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
    let grit = value_noise(direction * 48.0 + Vec3::new(4.0, 7.0, 2.0));
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

pub(super) fn shade_rocky_planet(input: &ShadeInput) -> Surface {
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
