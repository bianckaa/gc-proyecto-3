use super::*;

pub(super) fn shade_core_planet(input: &ShadeInput) -> Surface {
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
