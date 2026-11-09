use super::{Aim, CombatEvents, Targets, WeaponKind, WeaponStats, MUZZLE_DISTANCE};
use crate::engine::math::Vec3;
use crate::renderer::framebuffer::Color;
use crate::renderer::Renderer;

const FUSE: f32 = 1.6;
const DRAG: f32 = 1.2;
const CONTACT_RADIUS: f32 = 0.5;
const DROP_HEIGHT: f32 = -0.4;
const BOMB_SIZE: f32 = 0.45;
const BLINK_RATE: f32 = 6.0;
const COLOR: Color = Color::new(255, 140, 40);
const WARNING: Color = Color::new(255, 240, 200);

pub fn stats(step: f32) -> WeaponStats {
    WeaponStats {
        damage: 35.0 + 12.0 * step,
        cooldown: 3.2 - 0.3 * step,
        size: 7.0 + 1.5 * step,
        knockback: 20.0,
        speed: 18.0,
        count: 1,
    }
}

struct Bomb {
    position: Vec3,
    velocity: Vec3,
    fuse: f32,
}

pub struct Bombs {
    timer: f32,
    armed: Vec<Bomb>,
}

impl Bombs {
    pub fn new() -> Self {
        Self {
            timer: 1.0,
            armed: Vec::new(),
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
        let Some(stats) = stats else {
            self.armed.clear();
            return;
        };
        self.timer -= delta_time;
        if self.timer <= 0.0 {
            self.timer = stats.cooldown;
            self.armed.push(Bomb {
                position: aim.origin
                    + aim.forward.normalize() * MUZZLE_DISTANCE
                    + aim.up.normalize() * DROP_HEIGHT,
                velocity: aim.forward.normalize() * stats.speed,
                fuse: FUSE,
            });
            events.shots.push(WeaponKind::Bombs);
        }
        let mut detonations = Vec::new();
        self.armed.retain_mut(|bomb| {
            let start = bomb.position;
            let end = start + bomb.velocity * delta_time;
            bomb.velocity = bomb.velocity * (-DRAG * delta_time).exp();
            bomb.fuse -= delta_time;
            let contact = targets.first_along(start, end, CONTACT_RADIUS);
            if let Some((time, _)) = contact {
                detonations.push(start + (end - start) * time);
                return false;
            }
            bomb.position = end;
            if bomb.fuse <= 0.0 {
                detonations.push(end);
                return false;
            }
            true
        });
        for center in detonations {
            for (target, position) in targets.within(center, stats.size) {
                let distance = (position - center).length();
                let falloff = 1.0 - 0.5 * (distance / stats.size).min(1.0);
                let away = (position - center).normalize() * stats.knockback;
                targets.hit(target, stats.damage * falloff, away, position, events);
            }
            events.explosions.push((center, stats.size));
        }
    }

    pub fn draw(&self, renderer: &mut Renderer, time: f32) {
        for bomb in &self.armed {
            let warning = bomb.fuse < FUSE * 0.4 && (time * BLINK_RATE * 2.0).sin() > 0.0;
            let color = if warning || (time * BLINK_RATE).sin() > 0.6 {
                WARNING
            } else {
                COLOR
            };
            let center = bomb.position;
            for axis in [
                Vec3::new(BOMB_SIZE, 0.0, 0.0),
                Vec3::new(0.0, BOMB_SIZE, 0.0),
                Vec3::new(0.0, 0.0, BOMB_SIZE),
            ] {
                renderer.draw_line_3d(center - axis, center + axis, color, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::run_once;
    use super::super::{WeaponKind, WeaponSystem};
    use crate::engine::math::Vec3;
    use crate::game::enemies::EnemySwarm;

    #[test]
    fn explosion_damages_a_whole_group() {
        let mut weapons = WeaponSystem::new();
        weapons.inventory.unlock(WeaponKind::Bombs);
        let mut enemies = EnemySwarm::new();
        for x in [-2.0, 0.0, 2.0] {
            enemies.spawn_at(Vec3::new(x, 4.0, -20.0), 40);
        }
        let mut explosions = 0;
        for _ in 0..240 {
            explosions += run_once(&mut weapons, &mut enemies, 1.0 / 60.0)
                .explosions
                .len();
        }
        assert!(explosions >= 1);
        assert!(enemies
            .enemies()
            .iter()
            .all(|enemy| enemy.health < enemy.max_health));
    }
}
