use super::*;

pub(super) fn shade_eta_player(input: &ShadeInput) -> Surface {
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

    // Dos propulsores emisivos en la parte trasera. El halo amplio evita que
    // la nave se pierda por completo en el dithering cuando queda de espaldas
    // al Sol, mientras que el pulso suave le da un poco de movimiento.
    let rear = smoothstep(0.74, 0.94, -point.z);
    let engine_distance = |center_x: f32| {
        let dx = point.x - center_x;
        let dy = point.y - 0.085;
        dx * dx + dy * dy
    };
    let distance = engine_distance(-0.105).min(engine_distance(0.105));
    let thruster_core = rear * (1.0 - smoothstep(0.0012, 0.0038, distance));
    let thruster_halo = rear * (1.0 - smoothstep(0.0038, 0.0120, distance));
    let thruster = thruster_core.max(thruster_halo * 0.58);
    let thruster_pulse = 0.82 + 0.18 * (input.time * 5.2).sin().abs();
    let thruster_glow = thruster * thruster_pulse;
    let thruster_blue = Vec3::new(0.08, 0.62, 1.0);
    color = mix(color, thruster_blue, thruster_glow * 0.92);

    let light = lit_intensity(input);
    let intensity =
        (light * (0.42 + panel * 0.46) + panel_detail * 0.08 + r2 * 0.16 + thruster_glow * 0.88)
            .clamp(0.16, 1.0);
    Surface { color, intensity }
}
