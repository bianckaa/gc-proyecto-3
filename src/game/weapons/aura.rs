use super::{ring_points, Aim, CombatEvents, Targets, WeaponKind, WeaponStats};
use crate::renderer::framebuffer::Color;
use crate::renderer::Renderer;

const RING_SEGMENTS: usize = 36;
const PULSE_SPEED: f32 = 4.0;
const PULSE_AMOUNT: f32 = 0.06;
const COLOR: Color = Color::new(80, 230, 255);
const DIM_COLOR: Color = Color::new(30, 110, 140);

pub fn stats(step: f32) -> WeaponStats {
    WeaponStats {
        level: 1,
        damage: 10.0 + 6.0 * step,
        cooldown: (0.5 - 0.06 * step).max(0.18),
        size: 6.0 + 1.2 * step,
        knockback: if step >= 9.0 { 36.0 } else { 6.0 + 1.5 * step },
        speed: 0.0,
        count: 1,
    }
}

pub struct Aura {
    timer: f32,
}

impl Aura {
    pub fn new() -> Self {
        Self { timer: 0.0 }
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
            return;
        };
        self.timer -= delta_time;
        if self.timer > 0.0 {
            return;
        }
        self.timer += stats.cooldown;
        let mut touched = false;
        for (target, position) in targets.enemies_within(aim.origin, stats.size) {
            let away = (position - aim.origin).normalize() * stats.knockback;
            targets.hit(target, stats.damage, away, position, events);
            touched = true;
        }
        if touched {
            events.shots.push(WeaponKind::Aura);
        }
    }

    pub fn draw(&self, renderer: &mut Renderer, aim: Aim, radius: f32, time: f32) {
        let pulse = 1.0 + (time * PULSE_SPEED).sin() * PULSE_AMOUNT;
        let u_axis = aim.forward.normalize();
        let v_axis = aim.right.normalize();
        let outer = ring_points(aim.origin, u_axis, v_axis, radius * pulse, RING_SEGMENTS);
        let inner = ring_points(
            aim.origin,
            u_axis,
            v_axis,
            radius * 0.7 * pulse,
            RING_SEGMENTS,
        );
        for segment in outer.windows(2).step_by(2) {
            renderer.draw_line_3d(segment[0], segment[1], COLOR, false);
        }
        for segment in inner.windows(2).skip(1).step_by(3) {
            renderer.draw_line_3d(segment[0], segment[1], DIM_COLOR, false);
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
    fn damages_only_enemies_inside_the_radius() {
        let mut weapons = WeaponSystem::new();
        weapons.inventory.unlock(WeaponKind::Aura);
        let mut enemies = EnemySwarm::new();
        enemies.spawn_at(Vec3::new(0.0, 3.0, 0.0), 1);
        enemies.spawn_at(Vec3::new(40.0, 0.0, 0.0), 1);
        run_once(&mut weapons, &mut enemies, 1.0 / 60.0);
        let near = enemies
            .enemies()
            .iter()
            .find(|enemy| enemy.position.y > 1.0)
            .unwrap();
        let far = enemies
            .enemies()
            .iter()
            .find(|enemy| enemy.position.x > 10.0)
            .unwrap();
        assert!(near.health < near.max_health);
        assert_eq!(far.health, far.max_health);
    }

    #[test]
    fn damage_and_dps_increase_at_every_level() {
        let levels: Vec<_> = (0..10).map(|step| super::stats(step as f32)).collect();
        for pair in levels.windows(2) {
            assert!(pair[1].damage > pair[0].damage);
            assert!(pair[1].damage / pair[1].cooldown > pair[0].damage / pair[0].cooldown);
        }
        assert!(levels[9].knockback >= levels[8].knockback * 1.5);
    }
}
