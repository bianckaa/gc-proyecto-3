use super::*;

pub(super) fn shade_rocky(input: &ShadeInput) -> Surface {
    let point = input.fragment.local_position;
    let boulders = fbm(point * 5.0);
    let grit = value_noise(point * 28.0);
    let roughness = 0.45 + boulders * 0.7 + (grit - 0.5) * 0.35;
    Surface {
        color: input.material.base_color * (0.8 + boulders * 0.3),
        intensity: lit_intensity(input) * roughness,
    }
}
