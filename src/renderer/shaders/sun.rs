use super::*;

pub(super) fn shade_sun(input: &ShadeInput) -> Surface {
    let direction = input.fragment.local_position.normalize();
    let time = input.time;
    let instability = (input.material.emissive.z * 4.5).clamp(0.0, 1.0);
    let blue_shift =
        ((input.material.base_color.z - input.material.base_color.x) * 1.2).clamp(0.0, 1.0);
    let time_scale = 1.0 + instability * 2.4;

    // Capa 1: flujo de plasma deformado para evitar que el ruido parezca estatico.
    let flow = Vec3::new(time * 0.18, -time * 0.12, time * 0.07) * time_scale;
    let coarse = fbm(direction * 3.2 + flow);
    let warped = direction * 5.0
        + flow
        + Vec3::new(coarse, coarse * 0.6, -coarse) * (2.4 + instability * 1.8);
    let turbulence = fbm(warped);

    // Capa 2: granulos pequenos que hierven a una velocidad diferente.
    let granules = value_noise(
        direction * (34.0 + instability * 12.0)
            + Vec3::new(-time * 0.9, time * 0.7, time * 0.24) * time_scale,
    );

    // Capa 3: filamentos brillantes formados por crestas del campo turbulento.
    let filaments = (1.0 - ((turbulence - 0.52).abs() * 7.5)).clamp(0.0, 1.0);

    // Capa 4: erupciones locales con ciclos independientes sobre la superficie.
    let eruption_time = time * time_scale;
    let eruption = solar_eruption(direction, eruption_time, Vec3::new(0.76, 0.37, 0.53), 0.00)
        .max(solar_eruption(
            direction,
            eruption_time,
            Vec3::new(-0.48, 0.72, 0.50),
            0.37,
        ))
        .max(solar_eruption(
            direction,
            eruption_time,
            Vec3::new(0.20, -0.64, 0.74),
            0.71,
        ));

    let deep_orange = input.material.base_color * 0.86;
    let hot_yellow = mix(
        Vec3::new(1.0, 0.90, 0.32),
        Vec3::new(0.28, 0.72, 1.0),
        blue_shift,
    );
    let white_hot = mix(
        Vec3::new(1.0, 1.0, 0.92),
        Vec3::new(0.72, 0.91, 1.0),
        blue_shift,
    );
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
    let unstable_flash =
        instability * (0.5 + 0.30 * (time * 6.7).sin() + 0.20 * (time * 15.1 + 0.8).sin());
    let color = mix(color, white_hot, unstable_flash.max(0.0) * 0.22);
    let pulse = 0.92
        + 0.08 * (time * 1.7).sin()
        + instability * (0.10 * (time * 6.7).sin() + 0.05 * (time * 15.1 + 0.8).sin());
    let intensity = ((0.78 + turbulence * 0.48 + filaments * 0.38 + eruption * 0.70) * pulse)
        .max(0.28)
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
