use super::*;

pub(super) fn shade_droid_planet(input: &ShadeInput) -> Surface {
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
