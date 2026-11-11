use super::*;

pub(super) fn shade_gas_giant(input: &ShadeInput) -> Surface {
    let direction = input.fragment.local_position.normalize();
    let drift = Vec3::new(input.time * 0.05, 0.0, 0.0);
    let distortion = fbm(direction * 4.0 + drift) - 0.5;
    let latitude = direction.y + distortion * 0.25;
    let band = (latitude * 16.0).sin() * 0.5 + 0.5;
    Surface {
        color: mix(
            input.material.base_color * 0.55,
            input.material.base_color,
            band,
        ),
        intensity: lit_intensity(input) * (0.45 + 0.55 * band),
    }
}
