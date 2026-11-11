use super::*;

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

pub(super) fn shade_gas_planet(input: &ShadeInput) -> Surface {
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
        .clamp(GAS_NIGHT_DITHER_INTENSITY, 0.86);
    Surface { color, intensity }
}
