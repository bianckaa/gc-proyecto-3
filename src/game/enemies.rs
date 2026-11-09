use crate::engine::collision::{resolve_all, Collider};
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
const PLAYER_CONTACT_KNOCKBACK: f32 = 10.0;
const ENEMY_CONTACT_KNOCKBACK: f32 = 8.0;
const SEPARATION_PASSES: usize = 2;
const KNOCKBACK_DAMPING: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunEnemy {
    pub id: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
    level: u32,
    knockback_velocity: Vec3,
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
            knockback_velocity: Vec3::default(),
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
    pub player_knockback: Vec3,
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
        self.damage_with_knockback(id, amount, Vec3::default())
    }

    pub fn damage_with_knockback(&mut self, id: u64, amount: f32, impulse: Vec3) -> EnemyDamage {
        if !amount.is_finite() || amount <= 0.0 {
            return EnemyDamage::default();
        }
        let Some(index) = self.enemies.iter().position(|enemy| enemy.id == id) else {
            return EnemyDamage::default();
        };
        self.enemies[index].health = (self.enemies[index].health - amount).max(0.0);
        if impulse.x.is_finite() && impulse.y.is_finite() && impulse.z.is_finite() {
            self.enemies[index].knockback_velocity =
                self.enemies[index].knockback_velocity + impulse;
        }
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
        obstacles: &[Collider],
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
            enemy.velocity = to_player.normalize() * MOVE_SPEED + enemy.knockback_velocity;
            enemy.position = enemy.position + enemy.velocity * delta_time;
            enemy.knockback_velocity =
                enemy.knockback_velocity * (-KNOCKBACK_DAMPING * delta_time).exp();
        }
        self.enemies
            .retain(|enemy| (enemy.position - player_position).length() <= DESPAWN_DISTANCE);

        for _ in 0..SEPARATION_PASSES {
            self.separate_enemies();
            self.resolve_obstacles(obstacles);
        }

        let minimum_contact_distance = ENEMY_RADIUS + player_radius;
        let can_hit = self.contact_timer <= 0.0;
        let mut touching = false;
        let mut player_knockback = Vec3::default();
        if can_hit {
            for enemy in &mut self.enemies {
                let offset = player_position - enemy.position;
                let distance = offset.length();
                if distance > minimum_contact_distance {
                    continue;
                }
                touching = true;
                let away_from_enemy = if distance > f32::EPSILON {
                    offset * (1.0 / distance)
                } else {
                    -fallback_direction(0, enemy.id)
                };
                player_knockback = player_knockback + away_from_enemy;
                enemy.knockback_velocity =
                    enemy.knockback_velocity - away_from_enemy * ENEMY_CONTACT_KNOCKBACK;
            }
        }
        self.separate_from_player(player_position, player_radius);
        self.resolve_obstacles(obstacles);
        if touching {
            self.contact_timer = CONTACT_COOLDOWN;
            let direction = player_knockback.normalize();
            EnemyUpdate {
                contact_damage: CONTACT_DAMAGE,
                player_knockback: direction * PLAYER_CONTACT_KNOCKBACK,
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

    fn separate_enemies(&mut self) {
        let minimum_distance = ENEMY_RADIUS * 2.0;
        for first in 0..self.enemies.len() {
            for second in (first + 1)..self.enemies.len() {
                let (left, right) = self.enemies.split_at_mut(second);
                let first_enemy = &mut left[first];
                let second_enemy = &mut right[0];
                let offset = second_enemy.position - first_enemy.position;
                let distance = offset.length();
                if distance >= minimum_distance {
                    continue;
                }
                let direction = if distance > f32::EPSILON {
                    offset * (1.0 / distance)
                } else {
                    fallback_direction(first_enemy.id, second_enemy.id)
                };
                let correction = direction * ((minimum_distance - distance) * 0.5);
                first_enemy.position = first_enemy.position - correction;
                second_enemy.position = second_enemy.position + correction;
            }
        }
    }

    fn separate_from_player(&mut self, player_position: Vec3, player_radius: f32) {
        let minimum_distance = ENEMY_RADIUS + player_radius;
        for enemy in &mut self.enemies {
            let offset = enemy.position - player_position;
            let distance = offset.length();
            if distance >= minimum_distance {
                continue;
            }
            let direction = if distance > f32::EPSILON {
                offset * (1.0 / distance)
            } else {
                fallback_direction(0, enemy.id)
            };
            enemy.position = player_position + direction * minimum_distance;
        }
    }

    fn resolve_obstacles(&mut self, obstacles: &[Collider]) {
        for enemy in &mut self.enemies {
            let (position, velocity, _) =
                resolve_all(enemy.position, enemy.velocity, ENEMY_RADIUS, obstacles, 0.0);
            enemy.position = position;
            enemy.velocity = velocity;
        }
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

fn fallback_direction(first_id: u64, second_id: u64) -> Vec3 {
    match first_id.wrapping_mul(31).wrapping_add(second_id) % 3 {
        0 => Vec3::new(1.0, 0.0, 0.0),
        1 => Vec3::new(0.0, 1.0, 0.0),
        _ => Vec3::new(0.0, 0.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_scales_with_player_level_and_preserves_damage_ratio() {
        let mut swarm = EnemySwarm::new();
        let id = swarm.spawn_at(Vec3::new(10.0, 0.0, 0.0), 1);
        swarm.damage(id, 5.0);
        swarm.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 3, &[], 0.1);

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
                &[],
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
                &[],
                delta_time,
            )
        };

        let first = update(&mut swarm, 0.01);
        assert_eq!(first.contact_damage, CONTACT_DAMAGE);
        assert!(first.player_knockback.length() > 0.0);
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

    #[test]
    fn enemies_are_separated_from_each_other_and_the_player() {
        let mut swarm = EnemySwarm::new();
        swarm.spawn_at(Vec3::default(), 1);
        swarm.spawn_at(Vec3::default(), 1);

        swarm.update(
            Vec3::default(),
            Vec3::new(0.0, 0.0, -1.0),
            0.9,
            1,
            &[],
            1.0 / 60.0,
        );

        let first = swarm.enemies()[0].position;
        let second = swarm.enemies()[1].position;
        assert!((first - second).length() >= ENEMY_RADIUS * 2.0 - 1e-4);
        for enemy in swarm.enemies() {
            assert!(enemy.position.length() >= ENEMY_RADIUS + 0.9 - 1e-4);
        }
    }

    #[test]
    fn enemies_cannot_enter_world_colliders() {
        let mut swarm = EnemySwarm::new();
        swarm.spawn_at(Vec3::new(0.0, 0.0, 5.0), 1);
        let obstacle = Collider::sphere(Vec3::default(), 10.0);

        swarm.update(
            Vec3::new(0.0, 0.0, 30.0),
            Vec3::new(0.0, 0.0, -1.0),
            0.9,
            1,
            &[obstacle],
            1.0 / 60.0,
        );

        assert!(swarm.enemies()[0].position.length() >= obstacle.radius + ENEMY_RADIUS - 1e-4);
    }

    #[test]
    fn weapon_impulse_knocks_enemy_back_and_decays() {
        let mut swarm = EnemySwarm::new();
        let id = swarm.spawn_at(Vec3::new(0.0, 0.0, -10.0), 1);
        swarm.damage_with_knockback(id, 1.0, Vec3::new(0.0, 0.0, -12.0));
        let initial_knockback = swarm.enemies()[0].knockback_velocity.length();

        swarm.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 1, &[], 0.1);

        assert!(swarm.enemies()[0].position.z < -10.0);
        assert!(swarm.enemies()[0].knockback_velocity.length() < initial_knockback);
    }
}
