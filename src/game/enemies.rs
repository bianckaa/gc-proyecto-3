use crate::engine::math::Vec3;
use crate::world::generation::SeededRandom;

pub const ENEMY_RADIUS: f32 = 1.0;
pub const ENEMY_RENDER_SCALE: f32 = 0.9;
pub const CONTACT_DAMAGE: f32 = 10.0;

const BASE_HEALTH: f32 = 20.0;
const HEALTH_PER_LEVEL: f32 = 0.35;
const MOVE_SPEED: f32 = 7.0;
const INITIAL_SPAWN_DELAY: f32 = 2.0;
const SPAWN_INTERVAL: f32 = 4.0;
const MAX_ACTIVE: usize = 8;
const MIN_SPAWN_DISTANCE: f32 = 75.0;
const MAX_SPAWN_DISTANCE: f32 = 90.0;
const DESPAWN_DISTANCE: f32 = 700.0;
const CONTACT_COOLDOWN: f32 = 0.75;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunEnemy {
    pub id: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
    level: u32,
}

impl RunEnemy {
    fn new(id: u64, position: Vec3, level: u32) -> Self {
        let max_health = health_for_level(level);
        Self {
            id,
            position,
            velocity: Vec3::default(),
            health: max_health,
            max_health,
            level,
        }
    }

    fn sync_level(&mut self, level: u32) {
        if self.level == level {
            return;
        }
        let ratio = if self.max_health > 0.0 {
            self.health / self.max_health
        } else {
            0.0
        };
        self.max_health = health_for_level(level);
        self.health = self.max_health * ratio;
        self.level = level;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnemyUpdate {
    pub contact_damage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EnemyDamage {
    pub destroyed: bool,
}

pub struct EnemySwarm {
    enemies: Vec<RunEnemy>,
    random: SeededRandom,
    spawn_timer: f32,
    contact_timer: f32,
    next_id: u64,
}

impl EnemySwarm {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
            random: SeededRandom::new(1),
            spawn_timer: INITIAL_SPAWN_DELAY,
            contact_timer: 0.0,
            next_id: 1,
        }
    }

    pub fn reset(&mut self, seed: u32) {
        self.enemies.clear();
        self.random = SeededRandom::new(seed ^ 0xA511_E9B3);
        self.spawn_timer = INITIAL_SPAWN_DELAY;
        self.contact_timer = 0.0;
        self.next_id = 1;
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

    pub fn spawn_at(&mut self, position: Vec3, player_level: u32) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.enemies.push(RunEnemy::new(id, position, player_level));
        id
    }

    pub fn damage(&mut self, id: u64, amount: f32) -> EnemyDamage {
        if !amount.is_finite() || amount <= 0.0 {
            return EnemyDamage::default();
        }
        let Some(index) = self.enemies.iter().position(|enemy| enemy.id == id) else {
            return EnemyDamage::default();
        };
        self.enemies[index].health = (self.enemies[index].health - amount).max(0.0);
        let destroyed = self.enemies[index].health <= 0.0;
        if destroyed {
            self.enemies.swap_remove(index);
        }
        EnemyDamage { destroyed }
    }

    pub fn update(
        &mut self,
        player_position: Vec3,
        player_forward: Vec3,
        player_radius: f32,
        player_level: u32,
        delta_time: f32,
    ) -> EnemyUpdate {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return EnemyUpdate::default();
        }

        self.spawn_timer -= delta_time;
        self.contact_timer = (self.contact_timer - delta_time).max(0.0);
        if self.spawn_timer <= 0.0 && self.enemies.len() < MAX_ACTIVE {
            self.spawn_near_player(player_position, player_forward, player_level);
            self.spawn_timer += SPAWN_INTERVAL;
        }

        for enemy in &mut self.enemies {
            enemy.sync_level(player_level);
            let to_player = player_position - enemy.position;
            enemy.velocity = to_player.normalize() * MOVE_SPEED;
            enemy.position = enemy.position + enemy.velocity * delta_time;
        }
        self.enemies
            .retain(|enemy| (enemy.position - player_position).length() <= DESPAWN_DISTANCE);

        let touching = self.enemies.iter().any(|enemy| {
            (enemy.position - player_position).length() <= ENEMY_RADIUS + player_radius
        });
        if touching && self.contact_timer <= 0.0 {
            self.contact_timer = CONTACT_COOLDOWN;
            EnemyUpdate {
                contact_damage: CONTACT_DAMAGE,
            }
        } else {
            EnemyUpdate::default()
        }
    }

    fn spawn_near_player(
        &mut self,
        player_position: Vec3,
        player_forward: Vec3,
        player_level: u32,
    ) {
        let jitter = self.random.unit_direction() * 0.65;
        let direction = (-player_forward.normalize() + jitter).normalize();
        let distance = self.random.range(MIN_SPAWN_DISTANCE, MAX_SPAWN_DISTANCE);
        self.spawn_at(player_position + direction * distance, player_level);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_scales_with_player_level_and_preserves_damage_ratio() {
        let mut swarm = EnemySwarm::new();
        let id = swarm.spawn_at(Vec3::new(10.0, 0.0, 0.0), 1);
        swarm.damage(id, 5.0);
        swarm.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 3, 0.1);

        let enemy = swarm.enemies()[0];
        assert_eq!(enemy.max_health, health_for_level(3));
        assert!((enemy.health / enemy.max_health - 0.75).abs() < 1e-4);
    }

    #[test]
    fn same_seed_repeats_spawn_positions() {
        let simulate = |seed| {
            let mut swarm = EnemySwarm::new();
            swarm.reset(seed);
            swarm.update(
                Vec3::default(),
                Vec3::new(0.0, 0.0, -1.0),
                0.9,
                1,
                INITIAL_SPAWN_DELAY,
            );
            swarm.enemies()[0].position
        };

        assert_eq!(simulate(42), simulate(42));
        assert_ne!(simulate(42), simulate(43));
    }

    #[test]
    fn contact_damage_has_a_global_invulnerability_window() {
        let mut swarm = EnemySwarm::new();
        swarm.spawn_at(Vec3::default(), 1);
        let update = |swarm: &mut EnemySwarm, delta_time| {
            swarm.update(
                Vec3::default(),
                Vec3::new(0.0, 0.0, -1.0),
                0.9,
                1,
                delta_time,
            )
        };

        assert_eq!(update(&mut swarm, 0.01).contact_damage, CONTACT_DAMAGE);
        assert_eq!(update(&mut swarm, 0.5).contact_damage, 0.0);
        assert_eq!(update(&mut swarm, 0.25).contact_damage, CONTACT_DAMAGE);
    }

    #[test]
    fn damage_removes_destroyed_enemy() {
        let mut swarm = EnemySwarm::new();
        let id = swarm.spawn_at(Vec3::default(), 1);

        assert!(!swarm.damage(id, 5.0).destroyed);
        assert!(swarm.damage(id, 100.0).destroyed);
        assert_eq!(swarm.active_count(), 0);
    }
}
