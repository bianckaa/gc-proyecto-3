use super::{
    advance_projectiles, Aim, CombatEvents, Projectile, Targets, WeaponKind, WeaponStats,
    MUZZLE_DISTANCE,
};
use crate::renderer::framebuffer::Color;
use crate::renderer::Renderer;

const WING_OFFSET: f32 = 0.58;
const WING_HEIGHT: f32 = 0.08;
const PROJECTILE_LIFETIME: f32 = 4.0;
const LINE_LENGTH: f32 = 2.4;
const COLOR: Color = Color::new(255, 32, 32);

pub fn stats(step: f32) -> WeaponStats {
    WeaponStats {
        damage: 10.0 + 3.0 * step,
        cooldown: 0.30 - 0.03 * step,
        size: 0.18,
        knockback: 12.0,
        speed: 52.0,
        count: 1,
    }
}

pub struct NormalGun {
    pub projectiles: Vec<Projectile>,
    timer: f32,
    side: f32,
}

impl NormalGun {
    pub fn new() -> Self {
        Self {
            projectiles: Vec::new(),
            timer: 0.0,
            side: -1.0,
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
            let direction = aim.forward.normalize();
            while self.timer <= 0.0 {
                let muzzle = aim.origin
                    + direction * MUZZLE_DISTANCE
                    + aim.right.normalize() * (WING_OFFSET * self.side)
                    + aim.up.normalize() * WING_HEIGHT;
                self.projectiles.push(Projectile {
                    position: muzzle,
                    velocity: direction * stats.speed,
                    radius: stats.size,
                    damage: stats.damage,
                    knockback: stats.knockback,
                    remaining: PROJECTILE_LIFETIME,
                    traveled: 0.0,
                });
                self.timer += stats.cooldown;
                self.side = -self.side;
                events.shots.push(WeaponKind::Normal);
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
    use super::super::WeaponSystem;
    use crate::engine::math::Vec3;
    use crate::game::enemies::EnemySwarm;

    #[test]
    fn fires_automatically_and_alternates_wings() {
        let mut weapons = WeaponSystem::new();
        let mut enemies = EnemySwarm::new();
        let events = run_once(&mut weapons, &mut enemies, 0.31);
        let projectiles: Vec<_> = weapons.projectiles().collect();
        assert_eq!(events.shots.len(), 2);
        assert!(projectiles[0].position.x < 0.0 && projectiles[1].position.x > 0.0);
        assert!(projectiles[0].velocity.z < 0.0);
    }

    #[test]
    fn two_hits_destroy_a_base_enemy_without_tunneling() {
        let mut weapons = WeaponSystem::new();
        let mut enemies = EnemySwarm::new();
        enemies.spawn_at(Vec3::new(0.0, 0.0, -6.0), 1);
        let mut destroyed = 0;
        for _ in 0..120 {
            destroyed += run_once(&mut weapons, &mut enemies, 1.0 / 60.0)
                .enemies_destroyed
                .len();
        }
        assert_eq!(destroyed, 1);
        assert_eq!(enemies.active_count(), 0);
    }
}
