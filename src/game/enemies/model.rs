use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyKind {
    #[default]
    Common,
    Resistant,
    Elite,
}

impl EnemyKind {
    pub fn health_multiplier(self) -> f32 {
        match self {
            EnemyKind::Common => 1.0,
            EnemyKind::Resistant => 2.5,
            EnemyKind::Elite => 8.0,
        }
    }

    pub fn time_growth(self) -> f32 {
        match self {
            EnemyKind::Common => 0.0,
            EnemyKind::Resistant => 1.0,
            EnemyKind::Elite => 1.5,
        }
    }

    pub fn scale(self) -> f32 {
        match self {
            EnemyKind::Common => 1.0,
            EnemyKind::Resistant => 1.35,
            EnemyKind::Elite => 1.875,
        }
    }

    pub fn contact_damage(self) -> f32 {
        match self {
            EnemyKind::Common => CONTACT_DAMAGE,
            EnemyKind::Resistant => CONTACT_DAMAGE * 1.5,
            EnemyKind::Elite => CONTACT_DAMAGE * 2.5,
        }
    }

    pub fn experience(self) -> u32 {
        match self {
            EnemyKind::Common => 8,
            EnemyKind::Resistant => 18,
            EnemyKind::Elite => 70,
        }
    }
}

pub fn health_for(kind: EnemyKind, level: u32, elapsed: f32) -> f32 {
    let progress = (elapsed / RUN_DURATION_SECONDS).clamp(0.0, 1.0);
    health_for_level(level) * kind.health_multiplier() * (1.0 + kind.time_growth() * progress)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunEnemy {
    pub id: u64,
    pub kind: EnemyKind,
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub(super) level: u32,
    pub(super) heading: Vec3,
    pub(super) knockback_velocity: Vec3,
    pub(super) arrival: f32,
    pub(super) stray_time: f32,
    pub(super) fire_timer: f32,
    pub(super) elapsed: f32,
}

impl RunEnemy {
    pub(super) fn new(id: u64, position: Vec3, level: u32) -> Self {
        Self::of_kind(id, position, level, EnemyKind::Common, 0.0)
    }

    pub(super) fn of_kind(
        id: u64,
        position: Vec3,
        level: u32,
        kind: EnemyKind,
        elapsed: f32,
    ) -> Self {
        let max_health = health_for(kind, level, elapsed);
        Self {
            id,
            kind,
            position,
            velocity: Vec3::default(),
            health: max_health,
            max_health,
            level,
            heading: Vec3::default(),
            knockback_velocity: Vec3::default(),
            arrival: ARRIVAL_DURATION,
            stray_time: 0.0,
            fire_timer: ELITE_FIRST_SHOT,
            elapsed,
        }
    }

    pub(super) fn arriving(
        id: u64,
        position: Vec3,
        heading: Vec3,
        level: u32,
        kind: EnemyKind,
        elapsed: f32,
    ) -> Self {
        Self {
            heading: heading.normalize(),
            arrival: 0.0,
            ..Self::of_kind(id, position, level, kind, elapsed)
        }
    }

    pub fn radius(&self) -> f32 {
        ENEMY_RADIUS * self.kind.scale()
    }

    pub fn health_ratio(&self) -> f32 {
        if self.max_health > 0.0 {
            (self.health / self.max_health).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn arrival_progress(&self) -> f32 {
        (self.arrival / ARRIVAL_DURATION).clamp(0.0, 1.0)
    }

    pub fn is_materialized(&self) -> bool {
        self.arrival >= ARRIVAL_DURATION
    }

    pub fn arrival_scale(&self) -> f32 {
        let remaining = 1.0 - self.arrival_progress();
        let eased = 1.0 - remaining * remaining * remaining;
        ARRIVAL_START_SCALE + (1.0 - ARRIVAL_START_SCALE) * eased
    }

    pub fn forward(&self) -> Vec3 {
        if self.heading.length() > f32::EPSILON {
            self.heading
        } else if self.velocity.length() > f32::EPSILON {
            self.velocity.normalize()
        } else {
            Vec3::new(0.0, 0.0, -1.0)
        }
    }

    pub(super) fn is_stray(&self) -> bool {
        self.stray_time > 0.0
    }

    pub(super) fn sync(&mut self, level: u32, elapsed: f32) {
        if self.level == level && (self.elapsed - elapsed).abs() < 1.0 {
            return;
        }
        let ratio = if self.max_health > 0.0 {
            self.health / self.max_health
        } else {
            0.0
        };
        self.max_health = health_for(self.kind, level, elapsed);
        self.health = self.max_health * ratio;
        self.level = level;
        self.elapsed = elapsed;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnemyUpdate {
    pub contact_damage: f32,
    pub player_knockback: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EnemyDamage {
    pub destroyed: bool,
    pub kind: EnemyKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerMotion {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
}
