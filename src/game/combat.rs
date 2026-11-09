use crate::engine::collision::Collider;
use crate::engine::math::Vec3;
use crate::game::enemies::{EnemySwarm, ENEMY_RADIUS};

pub const PROJECTILE_RADIUS: f32 = 0.18;
pub const PROJECTILE_LINE_LENGTH: f32 = 2.4;

const MUZZLE_DISTANCE: f32 = 1.4;
const WING_MUZZLE_OFFSET: f32 = 0.58;
const WING_MUZZLE_HEIGHT: f32 = 0.08;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalWeaponStats {
    pub damage: f32,
    pub fire_interval: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime: f32,
    pub knockback: f32,
}

impl Default for NormalWeaponStats {
    fn default() -> Self {
        Self {
            damage: 10.0,
            fire_interval: 0.30,
            projectile_speed: 52.0,
            projectile_lifetime: 4.0,
            knockback: 12.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projectile {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
    damage: f32,
    remaining: f32,
    traveled: f32,
}

impl Projectile {
    pub fn line_start(&self) -> Vec3 {
        let visible_length = self.traveled.min(PROJECTILE_LINE_LENGTH);
        self.position - self.velocity.normalize() * visible_length
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CombatUpdate {
    pub shots_fired: u32,
    pub enemies_destroyed: u32,
}

pub struct WeaponSystem {
    projectiles: Vec<Projectile>,
    fire_timer: f32,
    normal: NormalWeaponStats,
    next_muzzle_side: f32,
}

impl WeaponSystem {
    pub fn new() -> Self {
        Self {
            projectiles: Vec::new(),
            fire_timer: 0.0,
            normal: NormalWeaponStats::default(),
            next_muzzle_side: -1.0,
        }
    }

    pub fn reset(&mut self) {
        self.projectiles.clear();
        self.fire_timer = 0.0;
        self.normal = NormalWeaponStats::default();
        self.next_muzzle_side = -1.0;
    }

    pub fn projectiles(&self) -> &[Projectile] {
        &self.projectiles
    }

    pub fn normal_stats(&self) -> NormalWeaponStats {
        self.normal
    }

    pub fn set_normal_projectile_speed(&mut self, speed: f32) {
        if speed.is_finite() && speed > 0.0 {
            self.normal.projectile_speed = speed.clamp(1.0, 200.0);
        }
    }

    pub fn set_normal_fire_interval(&mut self, interval: f32) {
        if interval.is_finite() && interval > 0.0 {
            self.normal.fire_interval = interval.clamp(0.05, 5.0);
        }
    }

    pub fn update(
        &mut self,
        origin: Vec3,
        aim_direction: Vec3,
        ship_right: Vec3,
        ship_up: Vec3,
        enemies: &mut EnemySwarm,
        obstacles: &[Collider],
        delta_time: f32,
    ) -> CombatUpdate {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return CombatUpdate::default();
        }

        let mut result = CombatUpdate::default();
        let direction = aim_direction.normalize();
        self.fire_timer -= delta_time;
        if direction.length() > 0.5 {
            while self.fire_timer <= 0.0 {
                let muzzle = origin
                    + direction * MUZZLE_DISTANCE
                    + ship_right.normalize() * (WING_MUZZLE_OFFSET * self.next_muzzle_side)
                    + ship_up.normalize() * WING_MUZZLE_HEIGHT;
                self.projectiles.push(Projectile {
                    position: muzzle,
                    velocity: direction * self.normal.projectile_speed,
                    radius: PROJECTILE_RADIUS,
                    damage: self.normal.damage,
                    remaining: self.normal.projectile_lifetime,
                    traveled: 0.0,
                });
                self.fire_timer += self.normal.fire_interval;
                self.next_muzzle_side = -self.next_muzzle_side;
                result.shots_fired += 1;
            }
        }

        let mut survivors = Vec::with_capacity(self.projectiles.len());
        for mut projectile in self.projectiles.drain(..) {
            let start = projectile.position;
            let end = start + projectile.velocity * delta_time;
            let enemy_hit = enemies
                .enemies()
                .iter()
                .filter_map(|enemy| {
                    segment_sphere_hit(start, end, enemy.position, projectile.radius + ENEMY_RADIUS)
                        .map(|time| (time, enemy.id))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let obstacle_hit = obstacles
                .iter()
                .filter_map(|collider| {
                    segment_sphere_hit(
                        start,
                        end,
                        collider.center,
                        collider.radius + projectile.radius,
                    )
                })
                .min_by(f32::total_cmp);

            let hits_enemy_first = match (enemy_hit, obstacle_hit) {
                (Some((enemy_time, _)), Some(obstacle_time)) => enemy_time <= obstacle_time,
                (Some(_), None) => true,
                _ => false,
            };
            if hits_enemy_first {
                let (_, enemy_id) = enemy_hit.unwrap();
                let impulse = projectile.velocity.normalize() * self.normal.knockback;
                if enemies
                    .damage_with_knockback(enemy_id, projectile.damage, impulse)
                    .destroyed
                {
                    result.enemies_destroyed += 1;
                }
                continue;
            }
            if obstacle_hit.is_some() {
                continue;
            }

            projectile.position = end;
            projectile.traveled += (end - start).length();
            projectile.remaining -= delta_time;
            if projectile.remaining > 0.0 {
                survivors.push(projectile);
            }
        }
        self.projectiles = survivors;
        result
    }
}

impl Default for WeaponSystem {
    fn default() -> Self {
        Self::new()
    }
}

fn segment_sphere_hit(start: Vec3, end: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let segment = end - start;
    let length_squared = segment.dot(segment);
    if length_squared <= f32::EPSILON {
        return ((start - center).length() <= radius).then_some(0.0);
    }
    let time = ((center - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    let closest = start + segment * time;
    ((closest - center).length() <= radius).then_some(time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_weapon_fires_automatically_along_aim_direction() {
        let mut weapons = WeaponSystem::new();
        let mut enemies = EnemySwarm::new();
        let result = weapons.update(
            Vec3::default(),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            &mut enemies,
            &[],
            1.0 / 60.0,
        );

        assert_eq!(result.shots_fired, 1);
        assert_eq!(weapons.projectiles().len(), 1);
        assert!(weapons.projectiles()[0].velocity.z < 0.0);
    }

    #[test]
    fn continuous_collision_does_not_tunnel_through_enemy() {
        let mut weapons = WeaponSystem::new();
        let mut enemies = EnemySwarm::new();
        let enemy = enemies.spawn_at(Vec3::new(0.0, 0.0, -10.0), 1);
        weapons.projectiles.push(Projectile {
            position: Vec3::default(),
            velocity: Vec3::new(0.0, 0.0, -100.0),
            radius: PROJECTILE_RADIUS,
            damage: NormalWeaponStats::default().damage,
            remaining: 1.0,
            traveled: 0.0,
        });
        weapons.fire_timer = 10.0;

        weapons.update(
            Vec3::default(),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            &mut enemies,
            &[],
            0.2,
        );

        assert_eq!(enemies.enemies()[0].id, enemy);
        assert_eq!(enemies.enemies()[0].health, 10.0);
        assert!(weapons.projectiles().is_empty());
    }

    #[test]
    fn two_normal_hits_destroy_base_enemy() {
        let mut enemies = EnemySwarm::new();
        let enemy = enemies.spawn_at(Vec3::default(), 1);

        let damage = NormalWeaponStats::default().damage;
        assert!(!enemies.damage(enemy, damage).destroyed);
        assert!(enemies.damage(enemy, damage).destroyed);
    }

    #[test]
    fn obstacle_blocks_projectile_before_enemy() {
        let mut weapons = WeaponSystem::new();
        let mut enemies = EnemySwarm::new();
        enemies.spawn_at(Vec3::new(0.0, 0.0, -20.0), 1);
        weapons.projectiles.push(Projectile {
            position: Vec3::default(),
            velocity: Vec3::new(0.0, 0.0, -100.0),
            radius: PROJECTILE_RADIUS,
            damage: NormalWeaponStats::default().damage,
            remaining: 1.0,
            traveled: 0.0,
        });
        weapons.fire_timer = 10.0;
        let obstacle = Collider::sphere(Vec3::new(0.0, 0.0, -10.0), 2.0);

        weapons.update(
            Vec3::default(),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            &mut enemies,
            &[obstacle],
            0.3,
        );

        assert_eq!(enemies.enemies()[0].health, enemies.enemies()[0].max_health);
        assert!(weapons.projectiles().is_empty());
    }

    #[test]
    fn projectile_speed_and_fire_interval_are_runtime_configurable() {
        let mut weapons = WeaponSystem::new();
        let mut enemies = EnemySwarm::new();
        weapons.set_normal_projectile_speed(16.0);
        weapons.set_normal_fire_interval(0.8);

        weapons.update(
            Vec3::default(),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            &mut enemies,
            &[],
            1.0 / 60.0,
        );

        assert_eq!(weapons.normal_stats().projectile_speed, 16.0);
        assert_eq!(weapons.normal_stats().fire_interval, 0.8);
        assert!((weapons.projectiles()[0].velocity.length() - 16.0).abs() < 1e-4);
    }

    #[test]
    fn shots_alternate_between_both_wing_muzzles() {
        let mut weapons = WeaponSystem::new();
        let mut enemies = EnemySwarm::new();

        let result = weapons.update(
            Vec3::default(),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            &mut enemies,
            &[],
            0.31,
        );

        assert_eq!(result.shots_fired, 2);
        assert!(weapons.projectiles()[0].position.x < 0.0);
        assert!(weapons.projectiles()[1].position.x > 0.0);
    }
}
