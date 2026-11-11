use super::{
    advance_projectiles, Aim, CombatEvents, Projectile, Targets, WeaponKind, WeaponStats,
    MUZZLE_DISTANCE,
};
use crate::renderer::framebuffer::Color;
use crate::renderer::Renderer;

const SHOT_INTERVAL: f32 = 0.06;
const SPREAD: f32 = 0.025;
const PROJECTILE_LIFETIME: f32 = 2.5;
const LINE_LENGTH: f32 = 1.4;
const BARREL_HEIGHT: f32 = -0.25;
const COLOR: Color = Color::new(255, 190, 60);

pub fn stats(step: f32) -> WeaponStats {
    let early = step.min(4.0);
    let evolved = (step - 4.0).max(0.0);
    WeaponStats {
        level: 1,
        damage: 4.0 + 1.5 * early + 0.8 * evolved,
        cooldown: (2.4 - 0.2 * step).max(0.8),
        size: 0.14,
        knockback: 4.0,
        speed: 72.0,
        count: if step >= 9.0 { 0 } else { 6 + 2 * step as u32 },
    }
}

pub struct Gatling {
    pub projectiles: Vec<Projectile>,
    reload: f32,
    shot_timer: f32,
    remaining_in_burst: u32,
    spin: f32,
}

impl Gatling {
    pub fn new() -> Self {
        Self {
            projectiles: Vec::new(),
            reload: 1.0,
            shot_timer: 0.0,
            remaining_in_burst: 0,
            spin: 0.0,
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
            let continuous = stats.level >= 10;
            if continuous {
                self.remaining_in_burst = 0;
                self.reload = 0.0;
            } else if self.remaining_in_burst == 0 {
                self.reload -= delta_time;
                if self.reload <= 0.0 {
                    self.remaining_in_burst = stats.count;
                    self.shot_timer = 0.0;
                    self.reload = stats.cooldown;
                }
            }
            self.shot_timer -= delta_time;
            while (continuous || self.remaining_in_burst > 0) && self.shot_timer <= 0.0 {
                self.spin += 2.1;
                let wobble =
                    aim.right * (self.spin.cos() * SPREAD) + aim.up * (self.spin.sin() * SPREAD);
                let direction = (aim.forward.normalize() + wobble).normalize();
                self.projectiles.push(Projectile {
                    position: aim.origin
                        + aim.forward.normalize() * MUZZLE_DISTANCE
                        + aim.up.normalize() * BARREL_HEIGHT,
                    velocity: direction * stats.speed,
                    radius: stats.size,
                    damage: stats.damage,
                    knockback: stats.knockback,
                    remaining: PROJECTILE_LIFETIME,
                    traveled: 0.0,
                });
                if !continuous {
                    self.remaining_in_burst -= 1;
                }
                self.shot_timer += SHOT_INTERVAL;
                events.shots.push(WeaponKind::Gatling);
            }
        }
        advance_projectiles(&mut self.projectiles, targets, events, delta_time);
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        for projectile in &self.projectiles {
            renderer.draw_line_3d(
                projectile.line_start(LINE_LENGTH),
                projectile.position,
                COLOR,
                false,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::run_once;
    use super::super::{WeaponKind, WeaponSystem};
    use crate::game::enemies::EnemySwarm;

    #[test]
    fn fires_a_full_burst_and_then_reloads() {
        let mut weapons = WeaponSystem::new();
        weapons.inventory.unlock(WeaponKind::Gatling);
        let mut enemies = EnemySwarm::new();
        let burst = WeaponKind::Gatling.stats(1).count as usize;
        let mut gatling_shots = 0;
        for _ in 0..90 {
            gatling_shots += run_once(&mut weapons, &mut enemies, 1.0 / 60.0)
                .shots
                .iter()
                .filter(|kind| **kind == WeaponKind::Gatling)
                .count();
        }
        assert_eq!(gatling_shots, burst);
    }

    #[test]
    fn level_ten_fires_continuously_without_reload_pauses() {
        let mut weapons = WeaponSystem::new();
        weapons.inventory.unlock(WeaponKind::Gatling);
        while weapons.inventory.upgrade(WeaponKind::Gatling) {}
        let mut enemies = EnemySwarm::new();
        let mut total = 0;
        let mut silent_frames = 0;
        let mut longest_silence = 0;
        for _ in 0..240 {
            let shots = run_once(&mut weapons, &mut enemies, 1.0 / 60.0)
                .shots
                .iter()
                .filter(|kind| **kind == WeaponKind::Gatling)
                .count();
            total += shots;
            if shots == 0 {
                silent_frames += 1;
                longest_silence = longest_silence.max(silent_frames);
            } else {
                silent_frames = 0;
            }
        }
        assert!(total >= 60);
        assert!(longest_silence <= 4);
    }
}
