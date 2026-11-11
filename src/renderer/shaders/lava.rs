use super::*;

pub(super) fn shade_lava(input: &ShadeInput) -> Surface {
    let flow = Vec3::new(0.0, input.time * 0.3, input.time * 0.1);
    let field = fbm(input.fragment.local_position * 4.0 + flow);
    let crack = 1.0 - ((field - 0.5).abs() * 12.0).min(1.0);
    let crust = lit_intensity(input) * 0.3;
    Surface {
        color: mix(input.material.base_color, Vec3::new(1.0, 0.75, 0.2), crack),
        intensity: crust + crack,
    }
}
