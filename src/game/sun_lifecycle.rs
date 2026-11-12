use crate::engine::math::Vec3;
use crate::game::session::RUN_DURATION_SECONDS;

const FOLLOW_DEAD_ZONE: f32 = 300.0;
const FOLLOW_RESPONSE: f32 = 0.12;
const MAX_FOLLOW_SPEED: f32 = 48.0;
pub const FIRST_GROWTH_MARK: f32 = 5.0 * 60.0;
pub const FINAL_PHASE_START: f32 = 10.0 * 60.0;
const FINAL_RADIUS_MULTIPLIER: f32 = 3.0;
const MAX_INSTABILITY_OFFSET: f32 = 18.0;
const COLLAPSE_MIN_SUN: f32 = 0.08;

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

pub fn follow_offset(sun: Vec3, player: Vec3, delta_time: f32) -> Vec3 {
    if !delta_time.is_finite() || delta_time <= 0.0 {
        return Vec3::default();
    }
    let displacement = player - sun;
    let distance = displacement.length();
    if distance <= FOLLOW_DEAD_ZONE {
        return Vec3::default();
    }

    let excess = distance - FOLLOW_DEAD_ZONE;
    let smooth_step = excess * (1.0 - (-FOLLOW_RESPONSE * delta_time).exp());
    let step = smooth_step.min(MAX_FOLLOW_SPEED * delta_time).min(excess);
    displacement * (step / distance)
}

pub fn radius_multiplier(elapsed: f32) -> f32 {
    let elapsed = elapsed.clamp(0.0, RUN_DURATION_SECONDS);
    if elapsed <= FIRST_GROWTH_MARK {
        1.0 + elapsed / FIRST_GROWTH_MARK
    } else if elapsed <= FINAL_PHASE_START {
        2.0 + (elapsed - FIRST_GROWTH_MARK) / (FINAL_PHASE_START - FIRST_GROWTH_MARK)
    } else {
        FINAL_RADIUS_MULTIPLIER
    }
}

pub fn instability(elapsed: f32) -> f32 {
    if elapsed <= FIRST_GROWTH_MARK {
        return 0.0;
    }
    if elapsed < FINAL_PHASE_START {
        let buildup = (elapsed - FIRST_GROWTH_MARK) / (FINAL_PHASE_START - FIRST_GROWTH_MARK);
        return smoothstep(buildup) * 0.35;
    }
    let final_phase = (elapsed - FINAL_PHASE_START) / (RUN_DURATION_SECONDS - FINAL_PHASE_START);
    0.35 + smoothstep(final_phase) * 0.65
}

pub fn blue_shift(elapsed: f32) -> f32 {
    smoothstep((elapsed - FINAL_PHASE_START) / (RUN_DURATION_SECONDS - FINAL_PHASE_START))
}

pub fn collapse_contraction(progress: f32) -> f32 {
    let shrink = (progress.clamp(0.0, 1.0) / 0.55).clamp(0.0, 1.0);
    1.0 - (1.0 - COLLAPSE_MIN_SUN) * smoothstep(shrink)
}

pub fn black_hole_intensity(progress: f32) -> f32 {
    smoothstep((progress.clamp(0.0, 1.0) - 0.15) / 0.6)
}

pub fn collapse_blackout(progress: f32) -> f32 {
    smoothstep((progress.clamp(0.0, 1.0) - 0.68) / 0.32)
}

pub fn visual_pulse(elapsed: f32, time: f32) -> f32 {
    let instability = instability(elapsed);
    let pulse = (time * 2.7).sin() * 0.026
        + (time * 7.1 + 0.8).sin() * 0.013
        + (time * 13.3 + 2.1).sin() * 0.006;
    1.0 + pulse * instability
}

pub fn instability_offset(elapsed: f32, time: f32) -> Vec3 {
    let amount = instability(elapsed).powf(1.25) * MAX_INSTABILITY_OFFSET;
    if amount <= f32::EPSILON {
        return Vec3::default();
    }
    let x = (time * 1.31).sin() * 0.68 + (time * 4.73 + 0.4).sin() * 0.32;
    let y = (time * 1.79 + 1.2).sin() * 0.62 + (time * 5.41).sin() * 0.38;
    let z = (time * 1.07 + 2.4).sin() * 0.72 + (time * 3.97 + 0.9).sin() * 0.28;
    Vec3::new(x, y, z) * amount
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_stays_still_inside_dead_zone() {
        let offset = follow_offset(Vec3::default(), Vec3::new(0.0, 0.0, 290.0), 1.0);
        assert_eq!(offset, Vec3::default());
    }

    #[test]
    fn distant_sun_follows_smoothly_without_crossing_dead_zone() {
        let sun = Vec3::default();
        let player = Vec3::new(0.0, 0.0, 500.0);
        let offset = follow_offset(sun, player, 1.0);

        assert!(offset.z > 0.0);
        assert!(offset.length() <= MAX_FOLLOW_SPEED + f32::EPSILON);
        assert!((player - (sun + offset)).length() >= FOLLOW_DEAD_ZONE);
    }

    #[test]
    fn follow_motion_is_stable_at_different_frame_rates() {
        let simulate = |delta_time: f32, steps: usize| {
            let player = Vec3::new(0.0, 0.0, 320.0);
            let mut sun = Vec3::default();
            for _ in 0..steps {
                sun = sun + follow_offset(sun, player, delta_time);
            }
            sun
        };

        let at_30_fps = simulate(1.0 / 30.0, 30);
        let at_144_fps = simulate(1.0 / 144.0, 144);
        assert!((at_30_fps - at_144_fps).length() < 1e-3);
    }

    #[test]
    fn sun_keeps_up_with_sustained_player_boost_at_long_range() {
        const BOOST_SPEED: f32 = 36.0;
        const STEP: f32 = 1.0 / 60.0;
        let mut sun = Vec3::default();
        let mut player = Vec3::new(0.0, 0.0, 104.0);

        for _ in 0..(60 * 120) {
            player.z += BOOST_SPEED * STEP;
            sun = sun + follow_offset(sun, player, STEP);
        }

        let distance = (player - sun).length();
        assert!(distance >= FOLLOW_DEAD_ZONE);
        assert!(distance < 650.0, "distancia final: {distance}");
    }

    #[test]
    fn growth_hits_the_five_and_ten_minute_targets() {
        let start = radius_multiplier(0.0);
        let five_minutes = radius_multiplier(FIRST_GROWTH_MARK);
        let ten_minutes = radius_multiplier(FINAL_PHASE_START);
        let end = radius_multiplier(RUN_DURATION_SECONDS);

        assert_eq!(start, 1.0);
        assert_eq!(five_minutes, 2.0);
        assert_eq!(ten_minutes, 3.0);
        assert_eq!(end, 3.0);
    }

    #[test]
    fn final_phase_turns_blue_and_becomes_increasingly_unstable() {
        assert_eq!(blue_shift(FINAL_PHASE_START), 0.0);
        assert_eq!(blue_shift(RUN_DURATION_SECONDS), 1.0);
        assert_eq!(instability(FIRST_GROWTH_MARK), 0.0);
        assert!(instability(FINAL_PHASE_START) > 0.3);
        assert_eq!(instability(RUN_DURATION_SECONDS), 1.0);
        assert_eq!(instability_offset(FIRST_GROWTH_MARK, 10.0), Vec3::default());
        assert!(instability_offset(RUN_DURATION_SECONDS, 10.0).length() > 1.0);
    }

    #[test]
    fn visual_pulse_stays_bounded_around_the_physical_radius() {
        for step in 0..600 {
            let pulse = visual_pulse(RUN_DURATION_SECONDS, step as f32 / 60.0);
            assert!((0.94..=1.06).contains(&pulse));
        }
    }

    #[test]
    fn collapse_contracts_into_a_black_hole_and_finishes_in_black() {
        assert_eq!(collapse_contraction(0.0), 1.0);
        assert!((collapse_contraction(1.0) - COLLAPSE_MIN_SUN).abs() < 1e-6);
        assert_eq!(black_hole_intensity(0.0), 0.0);
        assert_eq!(black_hole_intensity(1.0), 1.0);
        assert_eq!(collapse_blackout(0.68), 0.0);
        assert_eq!(collapse_blackout(1.0), 1.0);
    }
}
