use crate::engine::math::Vec3;
use crate::game::session::RUN_DURATION_SECONDS;

const FOLLOW_DEAD_ZONE: f32 = 300.0;
const FOLLOW_RESPONSE: f32 = 0.12;
const MAX_FOLLOW_SPEED: f32 = 48.0;
const MAX_RADIUS_MULTIPLIER: f32 = 2.4;

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
    let progress = (elapsed / RUN_DURATION_SECONDS).clamp(0.0, 1.0);
    let growth = 0.65 * progress + 0.75 * progress.powi(4);
    (1.0 + growth).min(MAX_RADIUS_MULTIPLIER)
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
    fn growth_is_slow_first_and_accelerates_near_collapse() {
        let start = radius_multiplier(0.0);
        let middle = radius_multiplier(RUN_DURATION_SECONDS * 0.5);
        let late = radius_multiplier(RUN_DURATION_SECONDS * 0.9);
        let end = radius_multiplier(RUN_DURATION_SECONDS);

        assert_eq!(start, 1.0);
        assert!(middle > start);
        assert!(late - middle > middle - start);
        assert!((end - MAX_RADIUS_MULTIPLIER).abs() < 1e-5);
    }
}
