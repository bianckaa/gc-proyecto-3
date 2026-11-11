use super::*;

pub(super) fn shade_sun(input: &ShadeInput) -> Surface {
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
