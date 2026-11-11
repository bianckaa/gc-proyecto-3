use super::*;

pub(super) fn shade_toon(input: &ShadeInput) -> Surface {
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
