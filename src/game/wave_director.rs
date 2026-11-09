use crate::game::session::RUN_DURATION_SECONDS;

const INITIAL_SPAWN_DELAY: f32 = 2.0;
const START_SPAWN_INTERVAL: f32 = 3.5;
const END_SPAWN_INTERVAL: f32 = 0.45;
const START_MAX_ACTIVE: f32 = 6.0;
const END_MAX_ACTIVE: f32 = 48.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveSettings {
    pub spawn_interval: f32,
    pub max_active: usize,
    pub threat_level: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveDirector {
    spawn_timer: f32,
}

impl WaveDirector {
    pub fn new() -> Self {
        Self {
            spawn_timer: INITIAL_SPAWN_DELAY,
        }
    }

    pub fn reset(&mut self) {
        self.spawn_timer = INITIAL_SPAWN_DELAY;
    }

    pub fn settings(&self, elapsed: f32) -> WaveSettings {
        settings_at(elapsed)
    }

    pub fn update(&mut self, elapsed: f32, active: usize, delta_time: f32) -> usize {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return 0;
        }
        let settings = settings_at(elapsed);
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
