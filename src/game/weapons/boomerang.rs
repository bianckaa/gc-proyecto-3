use super::{Aim, CombatEvents, TargetId, Targets, WeaponKind, WeaponStats, MUZZLE_DISTANCE};
use crate::engine::math::Vec3;
use crate::engine::transform::rotate_around;
use crate::renderer::framebuffer::Color;
use crate::renderer::Renderer;

const BASE_RANGE: f32 = 28.0;
const RANGE_PER_LEVEL: f32 = 4.0;
const HIT_COOLDOWN: f32 = 0.35;
const CATCH_DISTANCE: f32 = 1.6;
const MAX_LIFETIME: f32 = 6.0;
const FAN_ANGLE: f32 = 0.22;
const SPIN_SPEED: f32 = 14.0;
const ARM_LENGTH: f32 = 0.9;
const COLOR: Color = Color::new(200, 255, 90);

pub fn stats(step: f32) -> WeaponStats {
    WeaponStats {
        damage: 14.0 + 4.0 * step,
        cooldown: 1.8 - 0.15 * step,
        size: 0.9,
        knockback: 10.0,
        speed: 34.0,
        count: 1 + u32::from(step >= 1.0) + u32::from(step >= 3.0),
    }
}

fn range(stats: &WeaponStats) -> f32 {
    BASE_RANGE + RANGE_PER_LEVEL * (stats.count as f32 - 1.0)
}

struct Boomerang {
    position: Vec3,
    direction: Vec3,
    traveled: f32,
    returning: bool,
    age: f32,
    recent_hits: Vec<(TargetId, f32)>,
}

pub struct Boomerangs {
    timer: f32,
    flying: Vec<Boomerang>,
}

impl Boomerangs {
    pub fn new() -> Self {
        Self {
            timer: 0.5,
            flying: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        stats: Option<WeaponStats>,
        aim: Aim,
        targets: &mut Targets,
        events: &mut CombatEvents,
        delta_time: f32,
    ) {
        if let Some(stats) = stats {
            self.timer -= delta_time;
            if self.timer <= 0.0 && self.flying.is_empty() {
                self.timer = stats.cooldown;
                let count = stats.count.max(1);
                let target_direction = targets
                    .nearest_enemy(aim.origin)
                    .map(|position| (position - aim.origin).normalize())
                    .filter(|direction| direction.length() > f32::EPSILON)
                    .unwrap_or_else(|| aim.forward.normalize());
                for index in 0..count {
                    let offset = (index as f32 - (count - 1) as f32 * 0.5) * FAN_ANGLE;
                    let direction = rotate_around(target_direction, aim.up.normalize(), offset);
                    self.flying.push(Boomerang {
                        position: aim.origin + direction * MUZZLE_DISTANCE,
                        direction,
                        traveled: 0.0,
                        returning: false,
                        age: 0.0,
                        recent_hits: Vec::new(),
                    });
                }
                events.shots.push(WeaponKind::Boomerang);
            }
        }
        let Some(stats) = stats else {
            self.flying.clear();
            return;
        };
        let reach = range(&stats);
        for boomerang in &mut self.flying {
            boomerang.age += delta_time;
            boomerang.recent_hits.retain_mut(|(_, cooldown)| {
                *cooldown -= delta_time;
                *cooldown > 0.0
            });
            let velocity = if boomerang.returning {
                (aim.origin - boomerang.position).normalize() * (stats.speed * 1.2)
            } else {
                boomerang.direction * stats.speed
            };
            let start = boomerang.position;
            let end = start + velocity * delta_time;
            if !boomerang.returning
                && targets.obstacles.iter().any(|collider| {
                    super::segment_sphere_hit(start, end, collider.center, collider.radius)
                        .is_some()
                })
            {
                boomerang.returning = true;
                continue;
            }
            boomerang.position = end;
            boomerang.traveled += (end - start).length();
            if boomerang.traveled >= reach {
                boomerang.returning = true;
            }
            for (target, position) in targets.within(boomerang.position, stats.size) {
                if boomerang.recent_hits.iter().any(|(id, _)| *id == target) {
                    continue;
                }
                let impulse = velocity.normalize() * stats.knockback;
                targets.hit(target, stats.damage, impulse, position, events);
                boomerang.recent_hits.push((target, HIT_COOLDOWN));
            }
        }
        self.flying.retain(|boomerang| {
            let caught =
                boomerang.returning && (boomerang.position - aim.origin).length() < CATCH_DISTANCE;
            !caught && boomerang.age < MAX_LIFETIME
        });
    }

    pub fn draw(&self, renderer: &mut Renderer, aim: Aim, time: f32) {
        let axis = aim.up.normalize();
        for boomerang in &self.flying {
            let angle = time * SPIN_SPEED + boomerang.traveled;
            let arm = rotate_around(aim.right.normalize(), axis, angle) * ARM_LENGTH;
            let cross = rotate_around(arm, axis, std::f32::consts::FRAC_PI_2);
            let center = boomerang.position;
            renderer.draw_line_3d(center - arm, center + arm, COLOR, false);
            renderer.draw_line_3d(center, center + cross, COLOR, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{aim_forward, run_once};
    use super::super::{WeaponKind, WeaponSystem};
    use crate::engine::math::Vec3;
    use crate::game::enemies::EnemySwarm;

    #[test]
    fn flies_out_hits_on_both_passes_and_comes_back() {
        let mut weapons = WeaponSystem::new();
        weapons.inventory.unlock(WeaponKind::Boomerang);
        let mut enemies = EnemySwarm::new();
        let id = enemies.spawn_at(Vec3::new(0.0, 0.0, -14.0), 40);
        let mut thrown = false;
        let mut caught = false;
        for _ in 0..240 {
            run_once(&mut weapons, &mut enemies, 1.0 / 60.0);
            let flying = !weapons.boomerang.flying.is_empty();
            thrown |= flying;
            caught |= thrown && !flying;
        }
        let enemy = enemies
            .enemies()
            .iter()
            .find(|enemy| enemy.id == id)
            .unwrap();
        let single_hit = WeaponKind::Boomerang.stats(1).damage;
        assert!(enemy.max_health - enemy.health >= single_hit * 2.0 - 0.01);
        assert!(caught);
        assert_eq!(aim_forward().origin, Vec3::default());
    }

    #[test]
    fn launches_toward_the_nearest_enemy_outside_the_crosshair() {
        let mut weapons = WeaponSystem::new();
        weapons.inventory.unlock(WeaponKind::Boomerang);
        let mut enemies = EnemySwarm::new();
        let id = enemies.spawn_at(Vec3::new(12.0, 4.0, -12.0), 20);
        for _ in 0..120 {
            run_once(&mut weapons, &mut enemies, 1.0 / 60.0);
        }
        let enemy = enemies
            .enemies()
            .iter()
            .find(|enemy| enemy.id == id)
            .unwrap();
        assert!(enemy.health < enemy.max_health);
    }
}
