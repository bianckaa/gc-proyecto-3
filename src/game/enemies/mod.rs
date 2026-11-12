use crate::engine::collision::{resolve_all, Collider};
use crate::engine::math::Vec3;
use crate::engine::transform::rotate_around;
use crate::game::session::RUN_DURATION_SECONDS;
use crate::world::generation::SeededRandom;

mod model;
mod simulation;
mod spawning;
#[cfg(test)]
mod tests;

pub use model::{health_for, EnemyDamage, EnemyKind, EnemyUpdate, PlayerMotion, RunEnemy};

pub const ENEMY_RADIUS: f32 = 1.25;
pub const ENEMY_RENDER_SCALE: f32 = 1.125;
pub const CONTACT_DAMAGE: f32 = 10.0;
pub const ARRIVAL_DURATION: f32 = 0.6;

const BASE_HEALTH: f32 = 20.0;
const HEALTH_PER_LEVEL: f32 = 0.35;
const MOVE_SPEED: f32 = 18.0;
const CATCH_UP_SPEED: f32 = 60.0;
const CATCH_UP_START: f32 = 50.0;
const CATCH_UP_FULL: f32 = 180.0;
const MIN_SPAWN_DISTANCE: f32 = 105.0;
const MAX_SPAWN_DISTANCE: f32 = 135.0;
const SPAWN_SAFE_RADIUS: f32 = 100.0;
const SPAWN_JITTER: f32 = 0.65;
const SPAWN_LOOKAHEAD: f32 = 2.5;
const SPAWN_ATTEMPTS: usize = 8;
const SPAWN_SPREAD_STEP: f32 = 0.5;
const SPAWN_CLEARANCE: f32 = 4.0;
const FAST_SHIP_SPEED: f32 = 21.0;
const ENGAGE_DISTANCE: f32 = 200.0;
const RECYCLE_GRACE: f32 = 1.5;
const ARRIVAL_START_SCALE: f32 = 0.6;
const DESPAWN_DISTANCE: f32 = 700.0;
const CONTACT_COOLDOWN: f32 = 0.75;
const PLAYER_CONTACT_KNOCKBACK: f32 = 10.0;
const ENEMY_CONTACT_KNOCKBACK: f32 = 8.0;
const SEPARATION_PASSES: usize = 2;
const KNOCKBACK_DAMPING: f32 = 5.0;
const TURN_RATE_RADIANS: f32 = 1.5;

const ELITE_FIRE_RANGE: f32 = 95.0;
const ELITE_FIRE_COOLDOWN: f32 = 5.0;
const ELITE_FIRE_JITTER: f32 = 1.5;
const ELITE_FIRE_RETRY: f32 = 0.4;
const ELITE_FIRST_SHOT: f32 = 2.5;

pub struct EnemySwarm {
    enemies: Vec<RunEnemy>,
    random: SeededRandom,
    contact_timer: f32,
    next_id: u64,
    run_elapsed: f32,
}

impl EnemySwarm {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
            random: SeededRandom::new(1),
            contact_timer: 0.0,
            next_id: 1,
            run_elapsed: 0.0,
        }
    }

    pub fn reset(&mut self, seed: u32) {
        self.enemies.clear();
        self.random = SeededRandom::new(seed ^ 0xA511_E9B3);
        self.contact_timer = 0.0;
        self.next_id = 1;
        self.run_elapsed = 0.0;
    }

    pub fn clear(&mut self) {
        self.enemies.clear();
        self.contact_timer = 0.0;
    }

    pub fn enemies(&self) -> &[RunEnemy] {
        &self.enemies
    }

    pub fn active_count(&self) -> usize {
        self.enemies.len()
    }

    pub fn engaged_count(&self) -> usize {
        self.enemies
            .iter()
            .filter(|enemy| !enemy.is_stray())
            .count()
    }

    pub fn set_run_elapsed(&mut self, elapsed: f32) {
        if elapsed.is_finite() {
            self.run_elapsed = elapsed.max(0.0);
        }
    }

    pub fn count_kind(&self, kind: EnemyKind) -> usize {
        self.enemies
            .iter()
            .filter(|enemy| enemy.kind == kind)
            .count()
    }

    pub fn position_of(&self, id: u64) -> Option<Vec3> {
        self.enemies
            .iter()
            .find(|enemy| enemy.id == id)
            .map(|enemy| enemy.position)
    }
}

impl Default for EnemySwarm {
    fn default() -> Self {
        Self::new()
    }
}

pub fn health_for_level(level: u32) -> f32 {
    BASE_HEALTH * (1.0 + level.saturating_sub(1) as f32 * HEALTH_PER_LEVEL)
}

pub fn pursuit_speed(distance: f32) -> f32 {
    let t = ((distance - CATCH_UP_START) / (CATCH_UP_FULL - CATCH_UP_START)).clamp(0.0, 1.0);
    let eased = t * t * (3.0 - 2.0 * t);
    MOVE_SPEED + (CATCH_UP_SPEED - MOVE_SPEED) * eased
}
