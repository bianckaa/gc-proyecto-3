use crate::engine::collision::{resolve_all, Collider};
use crate::engine::math::Vec3;
use crate::engine::transform::rotate_around;
use crate::world::generation::SeededRandom;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunEnemy {
    pub id: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
    level: u32,
    heading: Vec3,
    knockback_velocity: Vec3,
    arrival: f32,
    stray_time: f32,
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
            heading: Vec3::default(),
            knockback_velocity: Vec3::default(),
            arrival: ARRIVAL_DURATION,
            stray_time: 0.0,
        }
    }

    fn arriving(id: u64, position: Vec3, heading: Vec3, level: u32) -> Self {
        Self {
            heading: heading.normalize(),
            arrival: 0.0,
            ..Self::new(id, position, level)
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

    fn is_stray(&self) -> bool {
        self.stray_time > 0.0
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerMotion {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
}

pub struct EnemySwarm {
    enemies: Vec<RunEnemy>,
    random: SeededRandom,
    contact_timer: f32,
    next_id: u64,
}

impl EnemySwarm {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
            random: SeededRandom::new(1),
            contact_timer: 0.0,
            next_id: 1,
        }
    }

    pub fn reset(&mut self, seed: u32) {
        self.enemies.clear();
        self.random = SeededRandom::new(seed ^ 0xA511_E9B3);
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

    pub fn engaged_count(&self) -> usize {
        self.enemies
            .iter()
            .filter(|enemy| !enemy.is_stray())
            .count()
    }

    pub fn spawn_at(&mut self, position: Vec3, player_level: u32) -> u64 {
        let id = self.allocate_id();
        self.enemies.push(RunEnemy::new(id, position, player_level));
        id
    }

    pub fn spawn_near_player(
        &mut self,
        player_position: Vec3,
        player_forward: Vec3,
        player_level: u32,
    ) -> u64 {
        let motion = PlayerMotion {
            position: player_position,
            velocity: Vec3::default(),
            forward: player_forward,
        };
        self.spawn_around(motion, player_level, &[], |_| false)
    }

    pub fn spawn_around(
        &mut self,
        motion: PlayerMotion,
        player_level: u32,
        obstacles: &[Collider],
        is_visible: impl Fn(Vec3) -> bool,
    ) -> u64 {
        let position = self.arrival_point(motion, obstacles, &is_visible);
        let id = self.allocate_id();
        let heading = (motion.position - position).normalize();
        self.enemies
            .push(RunEnemy::arriving(id, position, heading, player_level));
        id
    }

    pub fn position_of(&self, id: u64) -> Option<Vec3> {
        self.enemies
            .iter()
            .find(|enemy| enemy.id == id)
            .map(|enemy| enemy.position)
    }

    pub fn recycle_stragglers(
        &mut self,
        motion: PlayerMotion,
        obstacles: &[Collider],
        is_visible: impl Fn(Vec3) -> bool,
        capacity: usize,
        delta_time: f32,
    ) -> Vec<Vec3> {
        let mut arrivals = Vec::new();
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return arrivals;
        }
        for enemy in &mut self.enemies {
            let far = (enemy.position - motion.position).length() > ENGAGE_DISTANCE;
            if far && !is_visible(enemy.position) {
                enemy.stray_time += delta_time;
            } else {
                enemy.stray_time = 0.0;
            }
        }

        let mut index = 0;
        while index < self.enemies.len() {
            if self.enemies[index].stray_time < RECYCLE_GRACE {
                index += 1;
                continue;
            }
            if self.engaged_count() >= capacity {
                self.enemies.swap_remove(index);
                continue;
            }
            let position = self.arrival_point(motion, obstacles, &is_visible);
            let enemy = &mut self.enemies[index];
            enemy.position = position;
            enemy.velocity = Vec3::default();
            enemy.heading = (motion.position - position).normalize();
            enemy.knockback_velocity = Vec3::default();
            enemy.arrival = 0.0;
            enemy.stray_time = 0.0;
            arrivals.push(position);
            index += 1;
        }
        arrivals
    }

    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }

    fn arrival_point(
        &mut self,
        motion: PlayerMotion,
        obstacles: &[Collider],
        is_visible: &impl Fn(Vec3) -> bool,
    ) -> Vec3 {
        let haste = (motion.velocity.length() / FAST_SHIP_SPEED).clamp(0.0, 1.0);
        let anchor = motion.position + motion.velocity * SPAWN_LOOKAHEAD;
        let behind = -motion.forward.normalize();
        let mut visible_fallback = None;
        let mut last_candidate = motion.position + behind * MIN_SPAWN_DISTANCE;
        for attempt in 0..SPAWN_ATTEMPTS {
            let spread = SPAWN_JITTER + haste * 2.0 + attempt as f32 * SPAWN_SPREAD_STEP;
            let jitter = self.random.unit_direction() * spread;
            let mut direction = (behind * (1.0 - haste) + jitter).normalize();
            if direction.length() <= f32::EPSILON {
                direction = behind;
            }
            let distance = self.random.range(MIN_SPAWN_DISTANCE, MAX_SPAWN_DISTANCE);
            let candidate = anchor + direction * distance;
            if (candidate - motion.position).length() < SPAWN_SAFE_RADIUS {
                continue;
            }
            last_candidate = candidate;
            if !is_clear_of_obstacles(candidate, obstacles) {
                continue;
            }
            if !is_visible(candidate) {
                return candidate;
            }
            visible_fallback.get_or_insert(candidate);
        }
        visible_fallback.unwrap_or(last_candidate)
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
        _player_forward: Vec3,
        player_radius: f32,
        player_level: u32,
        obstacles: &[Collider],
        delta_time: f32,
    ) -> EnemyUpdate {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return EnemyUpdate::default();
        }

        self.contact_timer = (self.contact_timer - delta_time).max(0.0);

        for enemy in &mut self.enemies {
            enemy.sync_level(player_level);
            enemy.arrival = (enemy.arrival + delta_time).min(ARRIVAL_DURATION);
            let to_player = player_position - enemy.position;
            let speed = pursuit_speed(to_player.length());
            let desired_heading = to_player.normalize();
            enemy.heading = turn_toward(
                enemy.heading,
                desired_heading,
                TURN_RATE_RADIANS * delta_time,
            );
            enemy.velocity = enemy.heading * speed + enemy.knockback_velocity;
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
                if !enemy.is_materialized() {
                    continue;
                }
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
            if !enemy.is_materialized() {
                continue;
            }
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

pub fn pursuit_speed(distance: f32) -> f32 {
    let t = ((distance - CATCH_UP_START) / (CATCH_UP_FULL - CATCH_UP_START)).clamp(0.0, 1.0);
    let eased = t * t * (3.0 - 2.0 * t);
    MOVE_SPEED + (CATCH_UP_SPEED - MOVE_SPEED) * eased
}

fn turn_toward(current: Vec3, target: Vec3, maximum_angle: f32) -> Vec3 {
    if target.length() <= f32::EPSILON {
        return current;
    }
    if current.length() <= f32::EPSILON {
        return target.normalize();
    }
    let current = current.normalize();
    let target = target.normalize();
    let angle = current.dot(target).clamp(-1.0, 1.0).acos();
    if angle <= maximum_angle || angle <= f32::EPSILON {
        return target;
    }
    let mut axis = current.cross(target);
    if axis.length() <= f32::EPSILON {
        let reference = if current.y.abs() < 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        axis = current.cross(reference);
    }
    rotate_around(current, axis.normalize(), maximum_angle.min(angle)).normalize()
}

fn is_clear_of_obstacles(position: Vec3, obstacles: &[Collider]) -> bool {
    obstacles.iter().all(|obstacle| {
        (position - obstacle.center).length() > obstacle.radius + ENEMY_RADIUS + SPAWN_CLEARANCE
    })
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
            swarm.spawn_near_player(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 1);
            swarm.enemies()[0].position
        };

        assert_eq!(simulate(42), simulate(42));
        assert_ne!(simulate(42), simulate(43));
    }

    #[test]
    fn procedural_spawns_respect_the_player_safe_radius() {
        let mut swarm = EnemySwarm::new();
        swarm.reset(11);
        let motion = cruising(Vec3::new(40.0, -10.0, 25.0), 36.0);

        for _ in 0..64 {
            let id = swarm.spawn_around(motion, 1, &[], |_| false);
            let position = swarm.position_of(id).unwrap();
            assert!((position - motion.position).length() >= SPAWN_SAFE_RADIUS);
        }
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
        swarm.enemies[0].position = Vec3::default();
        swarm.enemies[0].heading = Vec3::default();
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

    fn cruising(position: Vec3, speed: f32) -> PlayerMotion {
        PlayerMotion {
            position,
            velocity: Vec3::new(0.0, 0.0, -speed),
            forward: Vec3::new(0.0, 0.0, -1.0),
        }
    }

    #[test]
    fn distant_offscreen_enemy_is_recycled_near_the_player_after_grace() {
        let mut swarm = EnemySwarm::new();
        let id = swarm.spawn_at(Vec3::new(0.0, 0.0, 400.0), 1);
        swarm.damage(id, 5.0);
        let motion = cruising(Vec3::default(), 14.0);

        let early = swarm.recycle_stragglers(motion, &[], |_| false, 10, RECYCLE_GRACE * 0.5);
        assert!(early.is_empty());
        assert_eq!(swarm.engaged_count(), 0);

        let arrivals = swarm.recycle_stragglers(motion, &[], |_| false, 10, RECYCLE_GRACE);
        assert_eq!(arrivals.len(), 1);
        let enemy = swarm.enemies()[0];
        assert_eq!(enemy.id, id);
        assert!((enemy.position - motion.position).length() <= ENGAGE_DISTANCE);
        assert!(!enemy.is_materialized());
        assert_eq!(enemy.health, enemy.max_health - 5.0);
        assert_eq!(swarm.engaged_count(), 1);
    }

    #[test]
    fn visible_distant_enemy_is_never_recycled() {
        let mut swarm = EnemySwarm::new();
        let position = Vec3::new(0.0, 0.0, -400.0);
        swarm.spawn_at(position, 1);

        let arrivals =
            swarm.recycle_stragglers(cruising(Vec3::default(), 14.0), &[], |_| true, 10, 10.0);

        assert!(arrivals.is_empty());
        assert_eq!(swarm.enemies()[0].position, position);
        assert_eq!(swarm.engaged_count(), 1);
    }

    #[test]
    fn stragglers_over_capacity_vanish_offscreen_instead_of_returning() {
        let mut swarm = EnemySwarm::new();
        swarm.spawn_at(Vec3::new(5.0, 0.0, 0.0), 1);
        swarm.spawn_at(Vec3::new(0.0, 0.0, 400.0), 1);

        let arrivals =
            swarm.recycle_stragglers(cruising(Vec3::default(), 14.0), &[], |_| false, 1, 2.0);

        assert!(arrivals.is_empty());
        assert_eq!(swarm.active_count(), 1);
    }

    #[test]
    fn fast_ship_gets_enemies_ahead_and_out_of_view() {
        let mut swarm = EnemySwarm::new();
        swarm.reset(7);
        let motion = cruising(Vec3::default(), 36.0);
        let in_view = |position: Vec3| {
            position.z < 0.0
                && position.x.abs() < -position.z * 0.4
                && position.y.abs() < -position.z * 0.4
        };

        for _ in 0..20 {
            let id = swarm.spawn_around(motion, 1, &[], in_view);
            let position = swarm.position_of(id).unwrap();
            assert!(!in_view(position));
        }
        let average_z: f32 = swarm
            .enemies()
            .iter()
            .map(|enemy| enemy.position.z)
            .sum::<f32>()
            / 20.0;
        assert!(average_z < -40.0);
    }

    #[test]
    fn spawn_points_avoid_planets() {
        let mut swarm = EnemySwarm::new();
        swarm.reset(3);
        let planet = Collider::sphere(Vec3::new(0.0, 0.0, 80.0), 60.0);

        for _ in 0..20 {
            let id = swarm.spawn_around(cruising(Vec3::default(), 0.0), 1, &[planet], |_| false);
            let position = swarm.position_of(id).unwrap();
            assert!((position - planet.center).length() > planet.radius + ENEMY_RADIUS);
        }
    }

    #[test]
    fn arriving_enemy_is_a_harmless_ghost_until_materialized() {
        let mut swarm = EnemySwarm::new();
        let motion = cruising(Vec3::default(), 0.0);
        swarm.spawn_around(motion, 1, &[], |_| false);
        swarm.enemies[0].position = Vec3::default();
        assert_eq!(swarm.enemies()[0].arrival_progress(), 0.0);
        assert_eq!(swarm.enemies()[0].arrival_scale(), ARRIVAL_START_SCALE);

        let ghost = swarm.update(Vec3::default(), motion.forward, 0.9, 1, &[], 0.1);
        assert_eq!(ghost.contact_damage, 0.0);

        swarm.update(
            Vec3::default(),
            motion.forward,
            0.9,
            1,
            &[],
            ARRIVAL_DURATION,
        );
        assert!(swarm.enemies()[0].is_materialized());
        assert_eq!(swarm.enemies()[0].arrival_scale(), 1.0);
        swarm.enemies[0].position = Vec3::default();
        swarm.enemies[0].heading = Vec3::default();
        let solid = swarm.update(Vec3::default(), motion.forward, 0.9, 1, &[], 0.01);
        assert_eq!(solid.contact_damage, CONTACT_DAMAGE);
    }

    #[test]
    fn pursuit_matches_cruise_and_rubber_bands_at_distance() {
        assert_eq!(pursuit_speed(10.0), MOVE_SPEED);
        assert!(pursuit_speed(90.0) > MOVE_SPEED);
        assert_eq!(pursuit_speed(500.0), CATCH_UP_SPEED);
        assert!(MOVE_SPEED < FAST_SHIP_SPEED);
        assert!(CATCH_UP_SPEED > FAST_SHIP_SPEED);
    }

    #[test]
    fn enemies_need_time_to_reverse_their_heading() {
        let mut swarm = EnemySwarm::new();
        swarm.spawn_at(Vec3::new(10.0, 0.0, 0.0), 1);
        swarm.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 1, &[], 0.1);
        assert!(swarm.enemies()[0].forward().x < 0.0);

        swarm.update(
            Vec3::new(30.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            0.9,
            1,
            &[],
            0.1,
        );

        assert!(swarm.enemies()[0].forward().x < 0.0);
    }

    #[test]
    fn common_enemy_is_twenty_five_percent_larger() {
        assert_eq!(ENEMY_RADIUS, 1.25);
        assert_eq!(ENEMY_RENDER_SCALE, 0.9 * 1.25);
    }

    #[test]
    fn weapon_impulse_resists_pursuit_and_decays() {
        let mut swarm = EnemySwarm::new();
        let mut control = EnemySwarm::new();
        let id = swarm.spawn_at(Vec3::new(0.0, 0.0, -10.0), 1);
        control.spawn_at(Vec3::new(0.0, 0.0, -10.0), 1);
        swarm.damage_with_knockback(id, 1.0, Vec3::new(0.0, 0.0, -12.0));
        let initial_knockback = swarm.enemies()[0].knockback_velocity.length();

        swarm.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 1, &[], 0.1);
        control.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 1, &[], 0.1);

        assert!(swarm.enemies()[0].position.z < control.enemies()[0].position.z);
        assert!(swarm.enemies()[0].knockback_velocity.length() < initial_knockback);
    }
}
