use super::{Aim, CombatEvents, Targets, WeaponKind, WeaponStats, MUZZLE_DISTANCE};
use crate::engine::math::Vec3;
use crate::renderer::framebuffer::Color;
use crate::renderer::Renderer;

const BEAM_DURATION: f32 = 0.35;
const FIRST_SHOT_DELAY: f32 = 1.5;
const CORE: Color = Color::new(255, 235, 245);
const GLOW: Color = Color::new(255, 70, 150);

pub fn stats(step: f32) -> WeaponStats {
    WeaponStats {
        damage: 30.0 + 12.0 * step,
        cooldown: 4.0 - 0.4 * step,
        size: 0.8 + 0.2 * step,
        knockback: 16.0,
        speed: 160.0,
        count: 1,
    }
}

struct Beam {
    start: Vec3,
    end: Vec3,
    side: Vec3,
    width: f32,
    life: f32,
}

pub struct Laser {
    timer: f32,
    beams: Vec<Beam>,
}

impl Laser {
    pub fn new() -> Self {
        Self {
            timer: FIRST_SHOT_DELAY,
            beams: Vec::new(),
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
        self.beams.retain_mut(|beam| {
            beam.life -= delta_time;
            beam.life > 0.0
        });
        let Some(stats) = stats else {
            return;
        };
        self.timer -= delta_time;
        if self.timer > 0.0 {
            return;
        }
        self.timer = stats.cooldown;
        let direction = aim.forward.normalize();
        let start = aim.origin + direction * MUZZLE_DISTANCE;
        let mut end = start + direction * stats.speed;
        let blocked = targets
            .obstacles
            .iter()
            .filter_map(|collider| {
                super::segment_sphere_hit(start, end, collider.center, collider.radius)
            })
            .min_by(f32::total_cmp);
        if let Some(time) = blocked {
            end = start + (end - start) * time;
        }
        for (target, impact) in targets.all_along(start, end, stats.size) {
            targets.hit(target, stats.damage, direction * stats.knockback, impact, events);
        }
        self.beams.push(Beam {
            start,
            end,
            side: aim.right.normalize(),
            width: stats.size * 0.35,
            life: BEAM_DURATION,
        });
        events.shots.push(WeaponKind::Laser);
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        for beam in &self.beams {
            let fade = beam.life / BEAM_DURATION;
            let offset = beam.side * (beam.width * fade);
            renderer.draw_line_3d(beam.start, beam.end, CORE, false);
            renderer.draw_line_3d(beam.start + offset, beam.end + offset, GLOW, fade < 0.5);
            renderer.draw_line_3d(beam.start - offset, beam.end - offset, GLOW, fade < 0.5);
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
    fn beam_pierces_every_enemy_in_line() {
        let mut weapons = WeaponSystem::new();
        weapons.inventory.unlock(WeaponKind::Laser);
        let mut enemies = EnemySwarm::new();
        for distance in [10.0, 30.0, 60.0] {
            enemies.spawn_at(Vec3::new(0.0, 0.0, -distance), 1);
        }
        let mut destroyed = 0;
        for _ in 0..100 {
            destroyed += run_once(&mut weapons, &mut enemies, 1.0 / 60.0)
                .enemies_destroyed
                .len();
        }
        assert_eq!(destroyed, 3);
    }
}
