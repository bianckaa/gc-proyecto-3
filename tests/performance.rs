use gc_proyecto_3::engine::math::Vec3;
use gc_proyecto_3::game::enemies::EnemySwarm;
use gc_proyecto_3::game::run_world::RunWorld;
use gc_proyecto_3::game::weapons::{Aim, Modifiers, Targets, WeaponKind, WeaponSystem};
use gc_proyecto_3::game::START_POSITION;
use gc_proyecto_3::world::SystemConfig;
use std::time::Instant;

const MAX_ENEMIES: usize = 48;
const FRAMES: usize = 600;
const FRAME_BUDGET_MS: f64 = 4.0;
const STEP: f32 = 1.0 / 60.0;

fn crowded_swarm() -> EnemySwarm {
    let mut enemies = EnemySwarm::new();
    for index in 0..MAX_ENEMIES {
        let angle = index as f32 * 0.37;
        let distance = 12.0 + (index % 8) as f32 * 3.0;
        enemies.spawn_at(
            Vec3::new(angle.cos() * distance, (index % 5) as f32 - 2.0, angle.sin() * distance),
            10,
        );
    }
    enemies
}

fn maxed_weapons() -> WeaponSystem {
    let mut weapons = WeaponSystem::new();
    for kind in WeaponKind::ALL {
        weapons.inventory.unlock(kind);
        while weapons.inventory.upgrade(kind) {}
    }
    weapons
}

#[test]
fn full_horde_with_every_weapon_fits_the_frame_budget() {
    let mut enemies = crowded_swarm();
    let mut weapons = maxed_weapons();
    let mut world = RunWorld::new();
    world.reset(42);
    let aim = Aim {
        origin: Vec3::default(),
        forward: Vec3::new(0.0, 0.0, -1.0),
        right: Vec3::new(1.0, 0.0, 0.0),
        up: Vec3::new(0.0, 1.0, 0.0),
    };
    let start = Instant::now();
    for frame in 0..FRAMES {
        world.update(Vec3::new(frame as f32 * 2.0, 0.0, 0.0));
        while enemies.active_count() < MAX_ENEMIES {
            enemies.spawn_near_player(Vec3::default(), aim.forward, 10);
        }
        enemies.update(Vec3::default(), aim.forward, 0.9, 10, &[], STEP);
        let mut targets = Targets {
            enemies: &mut enemies,
            planets: &mut world,
            obstacles: &[],
        };
        weapons.update(aim, &mut targets, Modifiers::default(), STEP);
    }
    let average_ms = start.elapsed().as_secs_f64() * 1000.0 / FRAMES as f64;
    println!("simulacion con {MAX_ENEMIES} enemigos y 6 armas: {average_ms:.3} ms por cuadro");
    assert!(average_ms < FRAME_BUDGET_MS, "{average_ms:.3} ms por cuadro");
}

#[test]
fn no_seed_spawns_a_planet_on_top_of_the_start_or_the_sun() {
    let sun_radius = SystemConfig::game().bodies[0].radius;
    for seed in 0..400u32 {
        let mut world = RunWorld::new();
        world.reset(seed.wrapping_mul(2_654_435_761));
        world.update(START_POSITION);
        for planet in world.planets() {
            let to_start = (planet.position - START_POSITION).length();
            let to_sun = planet.position.length();
            assert!(
                to_start > planet.radius + 25.0,
                "semilla {seed}: planeta a {to_start:.1} del inicio"
            );
            assert!(
                to_sun > planet.radius + sun_radius * 2.5,
                "semilla {seed}: planeta a {to_sun:.1} del sol"
            );
        }
    }
}
