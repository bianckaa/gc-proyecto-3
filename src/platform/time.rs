use std::time::Instant;

const MAX_FRAME_DELTA: f32 = 0.25;

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    last_tick: Instant,
    delta: f32,
    total: f64,
}

impl Clock {
    pub fn start() -> Self {
        Self {
            last_tick: Instant::now(),
            delta: 0.0,
            total: 0.0,
        }
    }

    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
        self.delta = elapsed.min(MAX_FRAME_DELTA);
        self.total += self.delta as f64;
        self.delta
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn total(&self) -> f32 {
        self.total as f32
    }
}
