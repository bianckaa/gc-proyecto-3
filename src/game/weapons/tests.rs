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
    weapons.update(
        aim_forward(),
        &mut targets,
        Modifiers::default(),
        delta_time,
    )
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
fn evolution_milestones_change_weapon_counts_and_knockback() {
    assert_eq!(WeaponKind::Normal.stats(4).count, 1);
    assert_eq!(WeaponKind::Normal.stats(5).count, 2);
    assert_eq!(WeaponKind::Laser.stats(4).count, 1);
    assert_eq!(WeaponKind::Laser.stats(5).count, 2);
    assert_eq!(WeaponKind::Laser.stats(10).count, 4);
    assert_eq!(WeaponKind::Bombs.stats(4).count, 1);
    assert_eq!(WeaponKind::Bombs.stats(5).count, 2);
    assert_eq!(WeaponKind::Bombs.stats(10).count, 3);
    assert_eq!(WeaponKind::Gatling.stats(10).count, 0);
    assert!(WeaponKind::Aura.stats(10).knockback > WeaponKind::Aura.stats(9).knockback);
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

#[test]
fn enemy_only_area_queries_ignore_planets() {
    let mut enemies = EnemySwarm::new();
    let enemy_id = enemies.spawn_at(Vec3::new(2.0, 0.0, 0.0), 1);
    let mut world = RunWorld::new();
    world.reset(7);
    world.update(Vec3::default());
    assert!(world.planets().next().is_some());
    let targets = Targets {
        enemies: &mut enemies,
        planets: &mut world,
        obstacles: &[],
    };
    assert_eq!(
        targets.enemies_within(Vec3::default(), 10_000.0),
        vec![(TargetId::Enemy(enemy_id), Vec3::new(2.0, 0.0, 0.0))]
    );
}
