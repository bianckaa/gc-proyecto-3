use crate::game::enemies::EnemyKind;
use crate::game::session::RUN_DURATION_SECONDS;

const INITIAL_SPAWN_DELAY: f32 = 2.0;
const START_SPAWN_INTERVAL: f32 = 3.5;
const END_SPAWN_INTERVAL: f32 = 0.45;
const START_MAX_ACTIVE: f32 = 6.0;
const END_MAX_ACTIVE: f32 = 48.0;
const RESISTANT_START: f32 = 45.0;
const MAX_RESISTANT_CHANCE: f32 = 0.4;
const ELITE_START: f32 = 120.0;
const START_ELITE_INTERVAL: f32 = 40.0;
const END_ELITE_INTERVAL: f32 = 18.0;
const START_MAX_ELITES: f32 = 1.0;
const END_MAX_ELITES: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveSettings {
    pub spawn_interval: f32,
    pub max_active: usize,
    pub threat_level: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveDirector {
    spawn_timer: f32,
    elite_timer: f32,
}

impl WaveDirector {
    pub fn new() -> Self {
        Self {
            spawn_timer: INITIAL_SPAWN_DELAY,
            elite_timer: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.spawn_timer = INITIAL_SPAWN_DELAY;
        self.elite_timer = 0.0;
    }

    pub fn settings(&self, elapsed: f32) -> WaveSettings {
        settings_at(elapsed)
    }

    pub fn choose_kind(&mut self, elapsed: f32, elites_active: usize, roll: f32) -> EnemyKind {
        let progress = (elapsed / RUN_DURATION_SECONDS).clamp(0.0, 1.0);
        let max_elites =
            (START_MAX_ELITES + (END_MAX_ELITES - START_MAX_ELITES) * progress).round() as usize;
        if elapsed >= ELITE_START && elites_active < max_elites && self.elite_timer <= 0.0 {
            self.elite_timer =
                START_ELITE_INTERVAL + (END_ELITE_INTERVAL - START_ELITE_INTERVAL) * progress;
            return EnemyKind::Elite;
        }
        if roll < resistant_chance(elapsed) {
            EnemyKind::Resistant
        } else {
            EnemyKind::Common
        }
    }

    pub fn update(&mut self, elapsed: f32, active: usize, delta_time: f32) -> usize {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return 0;
        }
        let settings = settings_at(elapsed);
        self.elite_timer = (self.elite_timer - delta_time).max(0.0);
        self.spawn_timer -= delta_time;
        let available = settings.max_active.saturating_sub(active);
        if available == 0 {
            self.spawn_timer = self.spawn_timer.max(0.0);
            return 0;
        }

        let mut spawn_count = 0;
        while self.spawn_timer <= 0.0 && spawn_count < available {
            spawn_count += 1;
            self.spawn_timer += settings.spawn_interval;
        }
        spawn_count
    }
}

impl Default for WaveDirector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn resistant_chance(elapsed: f32) -> f32 {
    if elapsed < RESISTANT_START {
        return 0.0;
    }
    let progress = ((elapsed - RESISTANT_START) / (RUN_DURATION_SECONDS - RESISTANT_START)).clamp(0.0, 1.0);
    MAX_RESISTANT_CHANCE * progress * progress * (3.0 - 2.0 * progress)
}

pub fn settings_at(elapsed: f32) -> WaveSettings {
    let progress = (elapsed / RUN_DURATION_SECONDS).clamp(0.0, 1.0);
    let intensity = progress.powf(1.2);
    WaveSettings {
        spawn_interval: START_SPAWN_INTERVAL
            + (END_SPAWN_INTERVAL - START_SPAWN_INTERVAL) * intensity,
        max_active: (START_MAX_ACTIVE + (END_MAX_ACTIVE - START_MAX_ACTIVE) * intensity).round()
            as usize,
        threat_level: 1 + (progress * 5.0).floor().min(4.0) as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elites_wait_their_turn_and_respect_the_limit() {
        let mut director = WaveDirector::new();
        assert_eq!(director.choose_kind(60.0, 0, 0.99), EnemyKind::Common);
        assert_eq!(director.choose_kind(200.0, 0, 0.99), EnemyKind::Elite);
        assert_ne!(director.choose_kind(200.0, 0, 0.99), EnemyKind::Elite);
        director.update(200.0, 0, 60.0);
        assert_eq!(director.choose_kind(200.0, 1, 0.99), EnemyKind::Common);
        assert_eq!(director.choose_kind(850.0, 1, 0.99), EnemyKind::Elite);
    }

    #[test]
    fn resistant_enemies_become_more_common_over_time() {
        assert_eq!(resistant_chance(30.0), 0.0);
        assert!(resistant_chance(300.0) > 0.0);
        assert!(resistant_chance(800.0) > resistant_chance(300.0));
        let mut director = WaveDirector::new();
        assert_eq!(director.choose_kind(800.0, 3, 0.0), EnemyKind::Resistant);
    }

    #[test]
    fn pressure_increases_monotonically_during_run() {
        let samples: Vec<_> = (0..=5)
            .map(|step| settings_at(RUN_DURATION_SECONDS * step as f32 / 5.0))
            .collect();
        for pair in samples.windows(2) {
            assert!(pair[1].spawn_interval <= pair[0].spawn_interval);
            assert!(pair[1].max_active >= pair[0].max_active);
            assert!(pair[1].threat_level >= pair[0].threat_level);
        }
        assert_eq!(samples[0].max_active, START_MAX_ACTIVE as usize);
        assert_eq!(samples[5].max_active, END_MAX_ACTIVE as usize);
    }

    #[test]
    fn first_enemy_appears_after_initial_delay() {
        let mut director = WaveDirector::new();
        assert_eq!(director.update(0.0, 0, INITIAL_SPAWN_DELAY - 0.01), 0);
        assert_eq!(director.update(0.0, 0, 0.01), 1);
    }

    #[test]
    fn active_cap_prevents_spawn_backlog() {
        let mut director = WaveDirector::new();
        let settings = director.settings(RUN_DURATION_SECONDS);
        assert_eq!(
            director.update(RUN_DURATION_SECONDS, settings.max_active, 30.0),
            0
        );
        assert_eq!(
            director.update(RUN_DURATION_SECONDS, settings.max_active - 1, 0.01),
            1
        );
    }
}
