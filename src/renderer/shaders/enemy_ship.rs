use super::*;

pub(super) fn shade_enemy_ship(input: &ShadeInput) -> Surface {
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
