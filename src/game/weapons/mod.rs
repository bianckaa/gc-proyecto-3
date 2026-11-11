mod aura;
mod bombs;
mod boomerang;
mod gatling;
mod laser;
mod normal;

use crate::engine::collision::Collider;
use crate::engine::math::Vec3;
use crate::game::enemies::{EnemyKind, EnemySwarm};
use crate::game::run_world::{DestroyedPlanet, RunWorld};
use crate::renderer::Renderer;

pub const MAX_WEAPON_LEVEL: u8 = 5;
pub const MUZZLE_DISTANCE: f32 = 1.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponKind {
    Normal,
    Gatling,
    Laser,
    Aura,
    Boomerang,
    Bombs,
}

impl WeaponKind {
    pub const ALL: [WeaponKind; 6] = [
        WeaponKind::Normal,
        WeaponKind::Gatling,
        WeaponKind::Laser,
        WeaponKind::Aura,
        WeaponKind::Boomerang,
        WeaponKind::Bombs,
    ];

    pub fn index(self) -> usize {
        match self {
            WeaponKind::Normal => 0,
            WeaponKind::Gatling => 1,
            WeaponKind::Laser => 2,
            WeaponKind::Aura => 3,
            WeaponKind::Boomerang => 4,
            WeaponKind::Bombs => 5,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Normal => "RAYO",
            WeaponKind::Gatling => "GATLING",
            WeaponKind::Laser => "LASER",
            WeaponKind::Aura => "AURA",
            WeaponKind::Boomerang => "BUMERAN",
            WeaponKind::Bombs => "BOMBAS",
        }
    }

    pub fn stats(self, level: u8) -> WeaponStats {
        let step = level.clamp(1, MAX_WEAPON_LEVEL) as f32 - 1.0;
        match self {
            WeaponKind::Normal => normal::stats(step),
            WeaponKind::Gatling => gatling::stats(step),
            WeaponKind::Laser => laser::stats(step),
            WeaponKind::Aura => aura::stats(step),
            WeaponKind::Boomerang => boomerang::stats(step),
            WeaponKind::Bombs => bombs::stats(step),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponStats {
    pub damage: f32,
    pub cooldown: f32,
    pub size: f32,
    pub knockback: f32,
    pub speed: f32,
    pub count: u32,
}

impl WeaponStats {
    pub fn modified(self, modifiers: Modifiers) -> Self {
        Self {
            damage: self.damage * modifiers.damage,
            cooldown: self.cooldown / modifiers.fire_rate.max(0.1),
            speed: self.speed * modifiers.projectile_speed,
            ..self
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "DANO {:.0}  RECARGA {:.2}S  TAMANO {:.1}",
            self.damage, self.cooldown, self.size
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Modifiers {
    pub damage: f32,
    pub fire_rate: f32,
    pub projectile_speed: f32,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            damage: 1.0,
            fire_rate: 1.0,
            projectile_speed: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inventory {
    levels: [u8; 6],
}

impl Inventory {
    pub fn new() -> Self {
        let mut levels = [0; 6];
        levels[WeaponKind::Normal.index()] = 1;
        Self { levels }
    }

    pub fn level(&self, kind: WeaponKind) -> u8 {
        self.levels[kind.index()]
    }

    pub fn owns(&self, kind: WeaponKind) -> bool {
        self.level(kind) > 0
    }

    pub fn is_maxed(&self, kind: WeaponKind) -> bool {
        self.level(kind) >= MAX_WEAPON_LEVEL
    }

    pub fn unlock(&mut self, kind: WeaponKind) -> bool {
        if self.owns(kind) {
            return self.upgrade(kind);
        }
        self.levels[kind.index()] = 1;
        true
    }

    pub fn upgrade(&mut self, kind: WeaponKind) -> bool {
        if !self.owns(kind) || self.is_maxed(kind) {
            return false;
        }
        self.levels[kind.index()] += 1;
        true
    }

    pub fn owned(&self) -> impl Iterator<Item = (WeaponKind, u8)> + '_ {
        WeaponKind::ALL
            .into_iter()
            .filter(|kind| self.owns(*kind))
            .map(|kind| (kind, self.level(kind)))
    }

    pub fn locked(&self) -> impl Iterator<Item = WeaponKind> + '_ {
        WeaponKind::ALL.into_iter().filter(|kind| !self.owns(*kind))
    }

    pub fn upgradable(&self) -> impl Iterator<Item = WeaponKind> + '_ {
        WeaponKind::ALL
            .into_iter()
            .filter(|kind| self.owns(*kind) && !self.is_maxed(*kind))
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetId {
    Enemy(u64),
    Planet(u64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aim {
    pub origin: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CombatEvents {
    pub shots: Vec<WeaponKind>,
    pub impacts: Vec<Vec3>,
    pub enemies_destroyed: Vec<(Vec3, EnemyKind)>,
    pub planets_destroyed: Vec<DestroyedPlanet>,
    pub explosions: Vec<(Vec3, f32)>,
    pub knockbacks: Vec<(Vec3, Vec3)>,
    pub damage: Vec<(Vec3, f32)>,
}

pub struct Targets<'a> {
    pub enemies: &'a mut EnemySwarm,
    pub planets: &'a mut RunWorld,
    pub obstacles: &'a [Collider],
}

pub fn segment_sphere_hit(start: Vec3, end: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let segment = end - start;
    let length_squared = segment.dot(segment);
    if length_squared <= f32::EPSILON {
        return ((start - center).length() <= radius).then_some(0.0);
    }
    let time = ((center - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    let closest = start + segment * time;
    ((closest - center).length() <= radius).then_some(time)
}

impl Targets<'_> {
    fn spheres(&self) -> Vec<(TargetId, Vec3, f32)> {
        let enemies = self
            .enemies
            .enemies()
            .iter()
            .map(|enemy| (TargetId::Enemy(enemy.id), enemy.position, enemy.radius()));
        let planets = self
            .planets
            .planets()
            .map(|planet| (TargetId::Planet(planet.id), planet.position, planet.radius));
        enemies.chain(planets).collect()
    }

    fn obstacle_time(&self, start: Vec3, end: Vec3, radius: f32) -> Option<f32> {
        self.obstacles
            .iter()
            .filter_map(|collider| {
                segment_sphere_hit(start, end, collider.center, collider.radius + radius)
            })
            .min_by(f32::total_cmp)
    }

    pub fn first_along(
        &self,
        start: Vec3,
        end: Vec3,
        radius: f32,
    ) -> Option<(f32, Option<TargetId>)> {
        let target = self
            .spheres()
            .into_iter()
            .filter_map(|(id, center, size)| {
                segment_sphere_hit(start, end, center, size + radius).map(|time| (time, Some(id)))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let obstacle = self
            .obstacle_time(start, end, radius)
            .map(|time| (time, None));
        match (target, obstacle) {
            (Some(target), Some(obstacle)) if obstacle.0 < target.0 => Some(obstacle),
            (Some(target), _) => Some(target),
            (None, obstacle) => obstacle,
        }
    }

    pub fn all_along(&self, start: Vec3, end: Vec3, radius: f32) -> Vec<(TargetId, Vec3)> {
        let limit = self.obstacle_time(start, end, radius).unwrap_or(1.0);
        self.spheres()
            .into_iter()
            .filter_map(|(id, center, size)| {
                segment_sphere_hit(start, end, center, size + radius)
                    .filter(|time| *time <= limit)
                    .map(|time| (id, start + (end - start) * time))
            })
            .collect()
    }

    pub fn within(&self, center: Vec3, radius: f32) -> Vec<(TargetId, Vec3)> {
        self.spheres()
            .into_iter()
            .filter(|(_, position, size)| (*position - center).length() <= radius + size)
            .map(|(id, position, _)| (id, position))
            .collect()
    }

    pub fn hit(
        &mut self,
        target: TargetId,
        damage: f32,
        impulse: Vec3,
        impact: Vec3,
        events: &mut CombatEvents,
    ) {
        events.impacts.push(impact);
        events.damage.push((impact, damage));
        match target {
            TargetId::Enemy(id) => {
                let position = self
                    .enemies
                    .enemies()
                    .iter()
                    .find(|enemy| enemy.id == id)
                    .map(|enemy| enemy.position);
                if impulse.length() > 0.01 {
                    events.knockbacks.push((position.unwrap_or(impact), impulse));
                }
                let result = self.enemies.damage_with_knockback(id, damage, impulse);
                if result.destroyed {
                    events
                        .enemies_destroyed
                        .push((position.unwrap_or(impact), result.kind));
                }
            }
            TargetId::Planet(id) => {
                if let Some(destroyed) = self.planets.damage_planet(id, damage) {
                    events.planets_destroyed.push(destroyed);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projectile {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
    pub damage: f32,
    pub knockback: f32,
    pub remaining: f32,
    pub traveled: f32,
}

impl Projectile {
    pub fn line_start(&self, length: f32) -> Vec3 {
        let visible = self.traveled.min(length);
        self.position - self.velocity.normalize() * visible
    }
}

fn advance_projectiles(
    projectiles: &mut Vec<Projectile>,
    targets: &mut Targets,
    events: &mut CombatEvents,
    delta_time: f32,
) {
    let mut survivors = Vec::with_capacity(projectiles.len());
    for mut projectile in projectiles.drain(..) {
        let start = projectile.position;
        let end = start + projectile.velocity * delta_time;
        match targets.first_along(start, end, projectile.radius) {
            Some((time, Some(target))) => {
                let impulse = projectile.velocity.normalize() * projectile.knockback;
                let impact = start + (end - start) * time;
                targets.hit(target, projectile.damage, impulse, impact, events);
                continue;
            }
            Some((time, None)) => {
                events.impacts.push(start + (end - start) * time);
                continue;
            }
            None => {}
        }
        projectile.position = end;
        projectile.traveled += (end - start).length();
        projectile.remaining -= delta_time;
        if projectile.remaining > 0.0 {
            survivors.push(projectile);
        }
    }
    *projectiles = survivors;
}

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
        self.laser.update(laser, aim, targets, &mut events, delta_time);
        self.aura.update(aura, aim, targets, &mut events, delta_time);
        self.boomerang
            .update(boomerang, aim, targets, &mut events, delta_time);
        self.bombs.update(bombs, aim, targets, &mut events, delta_time);
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
mod tests {
    use super::*;
    use crate::game::run_world::RunWorld;

    pub(super) fn aim_forward() -> Aim {
        Aim {
            origin: Vec3::default(),
            forward: Vec3::new(0.0, 0.0, -1.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
        }
    }

    pub(super) fn run_once(
        weapons: &mut WeaponSystem,
        enemies: &mut EnemySwarm,
        delta_time: f32,
    ) -> CombatEvents {
        let mut world = RunWorld::new();
        let mut targets = Targets {
            enemies,
            planets: &mut world,
            obstacles: &[],
        };
        weapons.update(aim_forward(), &mut targets, Modifiers::default(), delta_time)
    }

    #[test]
    fn inventory_starts_with_the_normal_ray_and_caps_levels() {
        let mut inventory = Inventory::new();
        assert_eq!(inventory.level(WeaponKind::Normal), 1);
        assert!(!inventory.owns(WeaponKind::Laser));
        assert!(inventory.unlock(WeaponKind::Laser));
        for _ in 0..10 {
            inventory.upgrade(WeaponKind::Laser);
        }
        assert_eq!(inventory.level(WeaponKind::Laser), MAX_WEAPON_LEVEL);
        assert!(!inventory.upgrade(WeaponKind::Laser));
        assert_eq!(inventory.locked().count(), 4);
    }

    #[test]
    fn higher_levels_and_modifiers_strengthen_every_weapon() {
        for kind in WeaponKind::ALL {
            let base = kind.stats(1);
            let top = kind.stats(MAX_WEAPON_LEVEL);
            assert!(top.damage > base.damage, "{}", kind.name());
            assert!(top.cooldown <= base.cooldown, "{}", kind.name());
        }
        let boosted = WeaponKind::Normal.stats(1).modified(Modifiers {
            damage: 2.0,
            fire_rate: 2.0,
            projectile_speed: 1.5,
        });
        assert_eq!(boosted.damage, WeaponKind::Normal.stats(1).damage * 2.0);
        assert!(boosted.cooldown < WeaponKind::Normal.stats(1).cooldown);
    }

    #[test]
    fn obstacle_blocks_a_target_behind_it() {
        let mut enemies = EnemySwarm::new();
        enemies.spawn_at(Vec3::new(0.0, 0.0, -20.0), 1);
        let mut world = RunWorld::new();
        let obstacle = [Collider::sphere(Vec3::new(0.0, 0.0, -10.0), 2.0)];
        let targets = Targets {
            enemies: &mut enemies,
            planets: &mut world,
            obstacles: &obstacle,
        };
        let hit = targets.first_along(Vec3::default(), Vec3::new(0.0, 0.0, -30.0), 0.1);
        assert!(matches!(hit, Some((_, None))));
    }
}
