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

#[derive(Debug, Clone, Copy)]
pub struct FixedTimestep {
    step: f32,
    max_steps: u32,
    accumulator: f32,
}

impl FixedTimestep {
    pub fn new(step: f32, max_steps: u32) -> Self {
        Self {
            step,
            max_steps,
            accumulator: 0.0,
        }
    }

    pub fn step(&self) -> f32 {
        self.step
    }

    pub fn advance(&mut self, frame_delta: f32) -> u32 {
        self.accumulator += frame_delta.max(0.0);
        let due_steps = (self.accumulator / self.step) as u32;
        let steps = due_steps.min(self.max_steps);
        self.accumulator -= steps as f32 * self.step;
        if self.accumulator >= self.step {
            self.accumulator %= self.step;
        }
        steps
    }

    pub fn alpha(&self) -> f32 {
        self.accumulator / self.step
    }
}
