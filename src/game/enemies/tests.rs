use super::*;

#[test]
fn tougher_kinds_have_more_health_that_grows_with_time() {
    let common = health_for(EnemyKind::Common, 5, 0.0);
    let resistant = health_for(EnemyKind::Resistant, 5, 0.0);
    let elite = health_for(EnemyKind::Elite, 5, 0.0);
    assert!(common < resistant && resistant < elite);
    assert_eq!(health_for(EnemyKind::Common, 5, 600.0), common);
    assert!(health_for(EnemyKind::Elite, 5, 600.0) > elite);
    assert!(EnemyKind::Common.scale() < EnemyKind::Resistant.scale());
    assert_eq!(EnemyKind::Elite.scale(), 1.875);
}

#[test]
fn health_rescales_when_time_passes_and_keeps_the_ratio() {
    let mut swarm = EnemySwarm::new();
    let id = swarm.spawn_kind_at(Vec3::new(0.0, 0.0, -50.0), 1, EnemyKind::Resistant);
    let before = swarm.enemies()[0].max_health;
    swarm.damage(id, before * 0.5);
    swarm.set_run_elapsed(600.0);
    swarm.update(
        Vec3::default(),
        Vec3::new(0.0, 0.0, -1.0),
        0.9,
        1,
        &[],
        1.0 / 60.0,
    );
    let enemy = swarm.enemies()[0];
    assert!(enemy.max_health > before);
    assert!((enemy.health_ratio() - 0.5).abs() < 0.01);
}

#[test]
fn elites_fire_in_range_and_retry_when_refused() {
    let mut swarm = EnemySwarm::new();
    swarm.spawn_kind_at(Vec3::new(0.0, 0.0, -30.0), 1, EnemyKind::Elite);
    swarm.spawn_kind_at(Vec3::new(0.0, 0.0, -300.0), 1, EnemyKind::Elite);
    swarm.spawn_at(Vec3::new(5.0, 0.0, -20.0), 1);
    let mut attempts = 0;
    for _ in 0..240 {
        swarm.elite_shots(Vec3::default(), 1.0 / 60.0, |_, origin| {
            assert!(origin.length() < ELITE_FIRE_RANGE);
            attempts += 1;
            false
        });
    }
    assert!(attempts >= 2);
    let mut accepted = 0;
    for _ in 0..240 {
        swarm.elite_shots(Vec3::default(), 1.0 / 60.0, |_, _| {
            accepted += 1;
            true
        });
    }
    assert_eq!(accepted, 1);
}

#[test]
fn elite_contact_hurts_more_and_reports_its_kind_when_destroyed() {
    let mut swarm = EnemySwarm::new();
    let id = swarm.spawn_kind_at(Vec3::new(0.0, 0.0, -1.5), 1, EnemyKind::Elite);
    let update = swarm.update(
        Vec3::default(),
        Vec3::new(0.0, 0.0, -1.0),
        0.9,
        1,
        &[],
        1.0 / 60.0,
    );
    assert_eq!(update.contact_damage, EnemyKind::Elite.contact_damage());
    let result = swarm.damage(id, 1_000_000.0);
    assert!(result.destroyed);
    assert_eq!(result.kind, EnemyKind::Elite);
}

#[test]
fn health_scales_with_player_level_and_preserves_damage_ratio() {
    let mut swarm = EnemySwarm::new();
    let id = swarm.spawn_at(Vec3::new(10.0, 0.0, 0.0), 1);
    swarm.damage(id, 5.0);
    swarm.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 3, &[], 0.1);

    let enemy = swarm.enemies()[0];
    assert_eq!(enemy.max_health, health_for_level(3));
    assert!((enemy.health / enemy.max_health - 0.75).abs() < 1e-4);
}

#[test]
fn same_seed_repeats_spawn_positions() {
    let simulate = |seed| {
        let mut swarm = EnemySwarm::new();
        swarm.reset(seed);
        swarm.spawn_near_player(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 1);
        swarm.enemies()[0].position
    };

    assert_eq!(simulate(42), simulate(42));
    assert_ne!(simulate(42), simulate(43));
}

#[test]
fn procedural_spawns_respect_the_player_safe_radius() {
    let mut swarm = EnemySwarm::new();
    swarm.reset(11);
    let motion = cruising(Vec3::new(40.0, -10.0, 25.0), 36.0);

    for _ in 0..64 {
        let id = swarm.spawn_around(motion, 1, &[], |_| false);
        let position = swarm.position_of(id).unwrap();
        assert!((position - motion.position).length() >= SPAWN_SAFE_RADIUS);
    }
}

#[test]
fn contact_damage_has_a_global_invulnerability_window() {
    let mut swarm = EnemySwarm::new();
    swarm.spawn_at(Vec3::default(), 1);
    let update = |swarm: &mut EnemySwarm, delta_time| {
        swarm.update(
            Vec3::default(),
            Vec3::new(0.0, 0.0, -1.0),
            0.9,
            1,
            &[],
            delta_time,
        )
    };

    let first = update(&mut swarm, 0.01);
    assert_eq!(first.contact_damage, CONTACT_DAMAGE);
    assert!(first.player_knockback.length() > 0.0);
    assert_eq!(update(&mut swarm, 0.5).contact_damage, 0.0);
    swarm.enemies[0].position = Vec3::default();
    swarm.enemies[0].heading = Vec3::default();
    assert_eq!(update(&mut swarm, 0.25).contact_damage, CONTACT_DAMAGE);
}

#[test]
fn damage_removes_destroyed_enemy() {
    let mut swarm = EnemySwarm::new();
    let id = swarm.spawn_at(Vec3::default(), 1);

    assert!(!swarm.damage(id, 5.0).destroyed);
    assert!(swarm.damage(id, 100.0).destroyed);
    assert_eq!(swarm.active_count(), 0);
}

#[test]
fn enemies_are_separated_from_each_other_and_the_player() {
    let mut swarm = EnemySwarm::new();
    swarm.spawn_at(Vec3::default(), 1);
    swarm.spawn_at(Vec3::default(), 1);

    swarm.update(
        Vec3::default(),
        Vec3::new(0.0, 0.0, -1.0),
        0.9,
        1,
        &[],
        1.0 / 60.0,
    );

    let first = swarm.enemies()[0].position;
    let second = swarm.enemies()[1].position;
    assert!((first - second).length() >= ENEMY_RADIUS * 2.0 - 1e-4);
    for enemy in swarm.enemies() {
        assert!(enemy.position.length() >= ENEMY_RADIUS + 0.9 - 1e-4);
    }
}

#[test]
fn enemies_cannot_enter_world_colliders() {
    let mut swarm = EnemySwarm::new();
    swarm.spawn_at(Vec3::new(0.0, 0.0, 5.0), 1);
    let obstacle = Collider::sphere(Vec3::default(), 10.0);

    swarm.update(
        Vec3::new(0.0, 0.0, 30.0),
        Vec3::new(0.0, 0.0, -1.0),
        0.9,
        1,
        &[obstacle],
        1.0 / 60.0,
    );

    assert!(swarm.enemies()[0].position.length() >= obstacle.radius + ENEMY_RADIUS - 1e-4);
}

fn cruising(position: Vec3, speed: f32) -> PlayerMotion {
    PlayerMotion {
        position,
        velocity: Vec3::new(0.0, 0.0, -speed),
        forward: Vec3::new(0.0, 0.0, -1.0),
    }
}

#[test]
fn distant_offscreen_enemy_is_recycled_near_the_player_after_grace() {
    let mut swarm = EnemySwarm::new();
    let id = swarm.spawn_at(Vec3::new(0.0, 0.0, 400.0), 1);
    swarm.damage(id, 5.0);
    let motion = cruising(Vec3::default(), 14.0);

    let early = swarm.recycle_stragglers(motion, &[], |_| false, 10, RECYCLE_GRACE * 0.5);
    assert!(early.is_empty());
    assert_eq!(swarm.engaged_count(), 0);

    let arrivals = swarm.recycle_stragglers(motion, &[], |_| false, 10, RECYCLE_GRACE);
    assert_eq!(arrivals.len(), 1);
    let enemy = swarm.enemies()[0];
    assert_eq!(enemy.id, id);
    assert!((enemy.position - motion.position).length() <= ENGAGE_DISTANCE);
    assert!(!enemy.is_materialized());
    assert_eq!(enemy.health, enemy.max_health - 5.0);
    assert_eq!(swarm.engaged_count(), 1);
}

#[test]
fn visible_distant_enemy_is_never_recycled() {
    let mut swarm = EnemySwarm::new();
    let position = Vec3::new(0.0, 0.0, -400.0);
    swarm.spawn_at(position, 1);

    let arrivals =
        swarm.recycle_stragglers(cruising(Vec3::default(), 14.0), &[], |_| true, 10, 10.0);

    assert!(arrivals.is_empty());
    assert_eq!(swarm.enemies()[0].position, position);
    assert_eq!(swarm.engaged_count(), 1);
}

#[test]
fn stragglers_over_capacity_vanish_offscreen_instead_of_returning() {
    let mut swarm = EnemySwarm::new();
    swarm.spawn_at(Vec3::new(5.0, 0.0, 0.0), 1);
    swarm.spawn_at(Vec3::new(0.0, 0.0, 400.0), 1);

    let arrivals =
        swarm.recycle_stragglers(cruising(Vec3::default(), 14.0), &[], |_| false, 1, 2.0);

    assert!(arrivals.is_empty());
    assert_eq!(swarm.active_count(), 1);
}

#[test]
fn fast_ship_gets_enemies_ahead_and_out_of_view() {
    let mut swarm = EnemySwarm::new();
    swarm.reset(7);
    let motion = cruising(Vec3::default(), 36.0);
    let in_view = |position: Vec3| {
        position.z < 0.0
            && position.x.abs() < -position.z * 0.4
            && position.y.abs() < -position.z * 0.4
    };

    for _ in 0..20 {
        let id = swarm.spawn_around(motion, 1, &[], in_view);
        let position = swarm.position_of(id).unwrap();
        assert!(!in_view(position));
    }
    let average_z: f32 = swarm
        .enemies()
        .iter()
        .map(|enemy| enemy.position.z)
        .sum::<f32>()
        / 20.0;
    assert!(average_z < -40.0);
}

#[test]
fn spawn_points_avoid_planets() {
    let mut swarm = EnemySwarm::new();
    swarm.reset(3);
    let planet = Collider::sphere(Vec3::new(0.0, 0.0, 80.0), 60.0);

    for _ in 0..20 {
        let id = swarm.spawn_around(cruising(Vec3::default(), 0.0), 1, &[planet], |_| false);
        let position = swarm.position_of(id).unwrap();
        assert!((position - planet.center).length() > planet.radius + ENEMY_RADIUS);
    }
}

#[test]
fn arriving_enemy_is_a_harmless_ghost_until_materialized() {
    let mut swarm = EnemySwarm::new();
    let motion = cruising(Vec3::default(), 0.0);
    swarm.spawn_around(motion, 1, &[], |_| false);
    swarm.enemies[0].position = Vec3::default();
    assert_eq!(swarm.enemies()[0].arrival_progress(), 0.0);
    assert_eq!(swarm.enemies()[0].arrival_scale(), ARRIVAL_START_SCALE);

    let ghost = swarm.update(Vec3::default(), motion.forward, 0.9, 1, &[], 0.1);
    assert_eq!(ghost.contact_damage, 0.0);

    swarm.update(
        Vec3::default(),
        motion.forward,
        0.9,
        1,
        &[],
        ARRIVAL_DURATION,
    );
    assert!(swarm.enemies()[0].is_materialized());
    assert_eq!(swarm.enemies()[0].arrival_scale(), 1.0);
    swarm.enemies[0].position = Vec3::default();
    swarm.enemies[0].heading = Vec3::default();
    let solid = swarm.update(Vec3::default(), motion.forward, 0.9, 1, &[], 0.01);
    assert_eq!(solid.contact_damage, CONTACT_DAMAGE);
}

#[test]
fn pursuit_matches_cruise_and_rubber_bands_at_distance() {
    assert_eq!(pursuit_speed(10.0), MOVE_SPEED);
    assert!(pursuit_speed(90.0) > MOVE_SPEED);
    assert_eq!(pursuit_speed(500.0), CATCH_UP_SPEED);
    assert!(MOVE_SPEED < FAST_SHIP_SPEED);
    assert!(CATCH_UP_SPEED > FAST_SHIP_SPEED);
}

#[test]
fn enemies_need_time_to_reverse_their_heading() {
    let mut swarm = EnemySwarm::new();
    swarm.spawn_at(Vec3::new(10.0, 0.0, 0.0), 1);
    swarm.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 1, &[], 0.1);
    assert!(swarm.enemies()[0].forward().x < 0.0);

    swarm.update(
        Vec3::new(30.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -1.0),
        0.9,
        1,
        &[],
        0.1,
    );

    assert!(swarm.enemies()[0].forward().x < 0.0);
}

#[test]
fn common_enemy_is_twenty_five_percent_larger() {
    assert_eq!(ENEMY_RADIUS, 1.25);
    assert_eq!(ENEMY_RENDER_SCALE, 0.9 * 1.25);
}

#[test]
fn weapon_impulse_resists_pursuit_and_decays() {
    let mut swarm = EnemySwarm::new();
    let mut control = EnemySwarm::new();
    let id = swarm.spawn_at(Vec3::new(0.0, 0.0, -10.0), 1);
    control.spawn_at(Vec3::new(0.0, 0.0, -10.0), 1);
    swarm.damage_with_knockback(id, 1.0, Vec3::new(0.0, 0.0, -12.0));
    let initial_knockback = swarm.enemies()[0].knockback_velocity.length();

    swarm.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 1, &[], 0.1);
    control.update(Vec3::default(), Vec3::new(0.0, 0.0, -1.0), 0.9, 1, &[], 0.1);

    assert!(swarm.enemies()[0].position.z < control.enemies()[0].position.z);
    assert!(swarm.enemies()[0].knockback_velocity.length() < initial_knockback);
}
