use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const RUN_DURATION_SECONDS: f32 = 15.0 * 60.0;

static RUN_SEQUENCE: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunState {
    #[default]
    Inactive,
    Playing,
    UpgradeRoulette,
    PlayerDestroyed,
    SolarCollapse,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunSession {
    state: RunState,
    elapsed: f32,
    seed: u32,
}

impl RunSession {
    pub fn new() -> Self {
        Self {
            state: RunState::Inactive,
            elapsed: 0.0,
            seed: 0,
        }
    }

    pub fn start(&mut self, seed: u32) {
        self.state = RunState::Playing;
        self.elapsed = 0.0;
        self.seed = seed;
    }

    pub fn start_random(&mut self) {
        self.start(generate_seed());
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Advances gameplay time and reports whether the Sun has just collapsed.
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.state != RunState::Playing || !delta_time.is_finite() || delta_time <= 0.0 {
            return false;
        }
        self.elapsed = (self.elapsed + delta_time).min(RUN_DURATION_SECONDS);
        if self.elapsed >= RUN_DURATION_SECONDS {
            self.state = RunState::SolarCollapse;
            return true;
        }
        false
    }

    pub fn state(&self) -> RunState {
        self.state
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn remaining(&self) -> f32 {
        (RUN_DURATION_SECONDS - self.elapsed).max(0.0)
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn begin_upgrade_roulette(&mut self) {
        if self.state == RunState::Playing {
            self.state = RunState::UpgradeRoulette;
        }
    }

    pub fn resume_playing(&mut self) {
        if self.state == RunState::UpgradeRoulette {
            self.state = RunState::Playing;
        }
    }

    pub fn destroy_player(&mut self) {
        if matches!(self.state, RunState::Playing | RunState::UpgradeRoulette) {
            self.state = RunState::PlayerDestroyed;
        }
    }

    pub fn show_summary(&mut self) {
        if matches!(
            self.state,
            RunState::PlayerDestroyed | RunState::SolarCollapse
        ) {
            self.state = RunState::Summary;
        }
    }
}

impl Default for RunSession {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate_seed() -> u32 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let time_seed = duration.as_secs() as u32 ^ duration.subsec_nanos().rotate_left(13);
    let seed = time_seed
        ^ RUN_SEQUENCE
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_mul(0x9E37_79B9);
    seed.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_run_starts_with_seed_and_full_duration() {
        let mut session = RunSession::new();
        session.start(42);

        assert_eq!(session.state(), RunState::Playing);
        assert_eq!(session.seed(), 42);
        assert_eq!(session.elapsed(), 0.0);
        assert_eq!(session.remaining(), RUN_DURATION_SECONDS);
    }

    #[test]
    fn timer_only_advances_while_playing() {
        let mut session = RunSession::new();
        session.start(7);
        session.update(12.0);
        session.begin_upgrade_roulette();
        session.update(30.0);

        assert_eq!(session.elapsed(), 12.0);
        session.resume_playing();
        session.update(3.0);
        assert_eq!(session.elapsed(), 15.0);
    }

    #[test]
    fn reaching_fifteen_minutes_starts_solar_collapse_once() {
        let mut session = RunSession::new();
        session.start(99);

        assert!(!session.update(RUN_DURATION_SECONDS - 0.25));
        assert!(session.update(0.25));
        assert_eq!(session.state(), RunState::SolarCollapse);
        assert_eq!(session.remaining(), 0.0);
        assert!(!session.update(1.0));
    }

    #[test]
    fn reset_removes_all_run_state() {
        let mut session = RunSession::new();
        session.start(123);
        session.update(8.0);
        session.reset();

        assert_eq!(session, RunSession::new());
    }
}
