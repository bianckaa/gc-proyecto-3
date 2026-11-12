mod aura;
mod bombs;
mod boomerang;
mod gatling;
mod inventory;
mod laser;
mod normal;
mod projectile;
mod stats;
mod targeting;

pub use inventory::{Inventory, WeaponKind};
use projectile::advance_projectiles;
pub use projectile::Projectile;
pub use stats::{Modifiers, WeaponStats};
pub use targeting::{segment_sphere_hit, Aim, CombatEvents, TargetId, Targets};

use crate::engine::collision::Collider;
use crate::engine::math::Vec3;
use crate::game::enemies::{EnemyKind, EnemySwarm};
use crate::game::run_world::{DestroyedPlanet, RunWorld};
use crate::renderer::Renderer;

pub const MAX_WEAPON_LEVEL: u8 = 10;
pub const MUZZLE_DISTANCE: f32 = 1.4;

pub struct WeaponSystem {
    pub inventory: Inventory,
    normal: normal::NormalGun,
    gatling: gatling::Gatling,
    laser: laser::Laser,
    aura: aura::Aura,
    boomerang: boomerang::Boomerangs,
    bombs: bombs::Bombs,
}

impl WeaponSystem {
    pub fn new() -> Self {
        Self {
            inventory: Inventory::new(),
            normal: normal::NormalGun::new(),
            gatling: gatling::Gatling::new(),
            laser: laser::Laser::new(),
            aura: aura::Aura::new(),
            boomerang: boomerang::Boomerangs::new(),
            bombs: bombs::Bombs::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn projectiles(&self) -> impl Iterator<Item = &Projectile> {
        self.normal
            .projectiles
            .iter()
            .chain(self.gatling.projectiles.iter())
    }

    pub fn update(
        &mut self,
        aim: Aim,
        targets: &mut Targets,
        modifiers: Modifiers,
        delta_time: f32,
    ) -> CombatEvents {
        let mut events = CombatEvents::default();
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return events;
        }
        let stats = |kind: WeaponKind| {
            let level = self.inventory.level(kind);
            (level > 0).then(|| kind.stats(level).modified(modifiers))
        };
        let (normal, gatling, laser, aura, boomerang, bombs) = (
            stats(WeaponKind::Normal),
            stats(WeaponKind::Gatling),
            stats(WeaponKind::Laser),
            stats(WeaponKind::Aura),
            stats(WeaponKind::Boomerang),
            stats(WeaponKind::Bombs),
        );
        self.normal
            .update(normal, aim, targets, &mut events, delta_time);
        self.gatling
            .update(gatling, aim, targets, &mut events, delta_time);
        self.laser
            .update(laser, aim, targets, &mut events, delta_time);
        self.aura
            .update(aura, aim, targets, &mut events, delta_time);
        self.boomerang
            .update(boomerang, aim, targets, &mut events, delta_time);
        self.bombs
            .update(bombs, aim, targets, &mut events, delta_time);
        events
    }

    pub fn draw(&self, renderer: &mut Renderer, aim: Aim, time: f32) {
        self.normal.draw(renderer);
        self.gatling.draw(renderer);
        self.laser.draw(renderer);
        let aura_level = self.inventory.level(WeaponKind::Aura);
        if aura_level > 0 {
            self.aura
                .draw(renderer, aim, WeaponKind::Aura.stats(aura_level).size, time);
        }
        self.boomerang.draw(renderer, aim, time);
        self.bombs.draw(renderer, time);
    }
}

impl Default for WeaponSystem {
    fn default() -> Self {
        Self::new()
    }
}

fn ring_points(center: Vec3, u_axis: Vec3, v_axis: Vec3, radius: f32, count: usize) -> Vec<Vec3> {
    (0..=count)
        .map(|index| {
            let angle = index as f32 / count as f32 * std::f32::consts::TAU;
            center + (u_axis * angle.cos() + v_axis * angle.sin()) * radius
        })
        .collect()
}

#[cfg(test)]
mod tests;
