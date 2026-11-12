use super::super::*;

#[test]
fn flight_speed_bonus_only_applies_in_game() {
    assert_eq!(flight_speed_multiplier(ProgramMode::Menu, 1.75), 1.0);
    assert_eq!(flight_speed_multiplier(ProgramMode::Game, 1.0), 1.5);
    assert_eq!(flight_speed_multiplier(ProgramMode::Game, 1.25), 1.875);
}

#[test]
fn crosshair_projects_a_point_ahead_of_the_ship_without_world_state() {
    let (mut game, mut camera) = test_game();
    let menu_position = game.crosshair_screen_position(&camera).unwrap();
    game.configure_mode(ProgramMode::Game, &mut camera);

    let forward_position = game.crosshair_screen_position(&camera).unwrap();
    game.ship.set_orientation(camera.right(), camera.up());
    let turned_position = game.crosshair_screen_position(&camera).unwrap();

    assert!(menu_position.0 >= CROSSHAIR_SCREEN_MARGIN);
    assert!(menu_position.1 >= CROSSHAIR_SCREEN_MARGIN);
    assert_ne!(forward_position, turned_position);
    assert!(turned_position.0 >= CROSSHAIR_SCREEN_MARGIN);
    assert!(turned_position.0 < FRAMEBUFFER_WIDTH as i32);
    assert!(turned_position.1 >= CROSSHAIR_SCREEN_MARGIN);
    assert!(turned_position.1 < FRAMEBUFFER_HEIGHT as i32);
}

fn test_game() -> (Game, Camera) {
    let mut assets = AssetManager::new();
    let mut camera = Camera::new(Vec3::default(), 16.0 / 9.0);
    let game = Game::new(&mut assets, &mut camera);
    (game, camera)
}

#[test]
fn game_starts_in_menu_and_sun_flash_switches_to_game() {
    let (mut game, mut camera) = test_game();
    let input = InputState::new();
    let mut effect = WarpEffect::new();
    assert_eq!(game.program_mode, ProgramMode::Menu);
    assert_eq!(
        game.hud_info().objective,
        Some("OBJETIVO: VUELA DENTRO DEL SOL PARA INICIAR")
    );

    game.ship.position = game.system.sun_position();
    game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);
    assert!(matches!(
        game.start_transition,
        StartTransition::Flashing { .. }
    ));

    game.update(&input, START_FLASH_IN_DURATION, &mut camera, &mut effect);
    assert_eq!(game.program_mode, ProgramMode::Game);
    assert_eq!(game.session.state(), RunState::Playing);
    assert_ne!(game.session.seed(), 0);
    assert_eq!(camera.far, CAMERA_FAR);
    assert_eq!(game.hud_info().objective, None);
    assert!((game.transition_flash() - 1.0).abs() < 0.001);
    assert_eq!(game.system.body_count(), 1);
    assert_eq!(game.system.body(0).name, "SOL");
}

#[test]
fn menu_planet_orbit_is_automatic() {
    let (mut game, mut camera) = test_game();
    let input = InputState::new();
    let mut effect = WarpEffect::new();
    let body = 1;
    let center = game.system.position(body);
    let radius = game.system.body(body).radius;
    game.ship.position = center + Vec3::new(radius * 2.0, 0.0, 0.0);

    game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);
    assert_eq!(game.ship.orbit_body(), Some(body));
    assert_eq!(game.ship.orbit_transition(), 0.0);

    for _ in 0..240 {
        game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);
    }
    let toward_planet = (game.system.position(body) - camera.position).normalize();
    assert!(camera.forward().dot(toward_planet) > 0.99);
}

#[test]
fn selected_seed_is_kept_by_the_run_session() {
    let (mut first, mut first_camera) = test_game();
    let (mut second, mut second_camera) = test_game();
    first.next_run_seed = 12_345;
    second.next_run_seed = 12_345;

    first.configure_mode(ProgramMode::Game, &mut first_camera);
    second.configure_mode(ProgramMode::Game, &mut second_camera);

    assert_eq!(first.session.seed(), 12_345);
    assert_eq!(first.session.seed(), second.session.seed());
    assert!(first.asteroids.is_empty());
    assert!(first.surfaces.is_empty());
    assert!(first.menu_patrols.is_empty());
    assert_eq!(first.run_enemies.active_count(), 0);
}

#[test]
fn nearby_damage_numbers_merge_and_fade() {
    let (mut game, mut camera) = test_game();
    game.configure_mode(ProgramMode::Game, &mut camera);
    game.push_damage_number(Vec3::new(0.0, 0.0, -10.0), 10.0);
    game.push_damage_number(Vec3::new(0.5, 0.0, -10.0), 5.0);
    game.push_damage_number(Vec3::new(30.0, 0.0, -10.0), 7.0);
    assert_eq!(game.damage_numbers.len(), 2);
    assert_eq!(game.damage_numbers[0].amount, 15.0);
    game.update_damage_numbers(DAMAGE_NUMBER_LIFE + 0.1);
    assert!(game.damage_numbers.is_empty());
}

#[test]
fn death_shows_results_and_enter_returns_to_menu() {
    let (mut game, mut camera) = test_game();
    game.configure_mode(ProgramMode::Game, &mut camera);
    assert!(game.damage_player(500.0));
    assert!(!game.ship_visible());
    let input = InputState::new();
    for _ in 0..200 {
        game.update_ending(&input, 1.0 / 60.0, &mut camera);
    }
    assert_eq!(game.session.state(), RunState::Summary);
    let summary = game.summary().unwrap();
    assert_eq!(summary.cause, "NAVE DESTRUIDA");
    assert_eq!(summary.weapons, vec!["RAYO NV1".to_owned()]);
    for _ in 0..(60 * 21) {
        game.update_ending(&input, 1.0 / 60.0, &mut camera);
    }
    assert_eq!(game.program_mode, ProgramMode::Menu);
}

#[test]
fn gameplay_sun_reaches_double_and_triple_its_initial_radius() {
    let (mut game, mut camera) = test_game();
    game.configure_mode(ProgramMode::Game, &mut camera);
    let initial = game.sun_base_radius;

    game.session.update(sun_lifecycle::FIRST_GROWTH_MARK);
    game.update_sun_lifecycle(1.0 / 60.0);
    assert!((game.system.body(0).radius - initial * 2.0).abs() < 1e-4);

    game.session
        .update(sun_lifecycle::FINAL_PHASE_START - sun_lifecycle::FIRST_GROWTH_MARK);
    game.update_sun_lifecycle(1.0 / 60.0);
    assert!((game.system.body(0).radius - initial * 3.0).abs() < 1e-4);
}

#[test]
fn solar_collapse_grows_a_black_hole_and_ends_the_run() {
    let (mut game, mut camera) = test_game();
    game.configure_mode(ProgramMode::Game, &mut camera);
    assert!(game.session.update(session::RUN_DURATION_SECONDS + 1.0));
    game.ending_cause = Some("EL SOL COLAPSO");
    let radius_before = game.system.body(0).radius;
    let input = InputState::new();
    for _ in 0..240 {
        game.update_ending(&input, 1.0 / 60.0, &mut camera);
        game.update_sun_lifecycle(1.0 / 60.0);
    }
    assert!(game.system.body(0).radius < radius_before);
    assert!(game.black_hole().1 > 0.5);
    for _ in 0..400 {
        game.update_ending(&input, 1.0 / 60.0, &mut camera);
    }
    assert_eq!(game.session.state(), RunState::Summary);
    assert_eq!(game.summary().unwrap().cause, "EL SOL COLAPSO");
    assert_eq!(game.solar_blackout(), 1.0);
    game.configure_mode(ProgramMode::Menu, &mut camera);
    assert_eq!(game.solar_blackout(), 0.0);
}

#[test]
fn level_ups_queue_roulettes_that_pause_the_run() {
    let (mut game, mut camera) = test_game();
    game.configure_mode(ProgramMode::Game, &mut camera);
    let levels = game.grant_player_experience(500);
    assert!(levels >= 2);
    let input = InputState::new();
    game.update_upgrades(&input, 1.0 / 60.0);
    assert_eq!(game.session.state(), RunState::UpgradeRoulette);
    assert!(!game.session.update(1.0));
    for _ in 0..levels {
        assert_eq!(game.session.state(), RunState::UpgradeRoulette);
        let roulette = game.roulette.as_mut().unwrap();
        roulette.update(5.0);
        let option = roulette.confirm().unwrap();
        game.confirm_upgrade(option);
    }
    assert_eq!(game.session.state(), RunState::Playing);
    assert_eq!(game.pending_upgrades, 0);
    assert!(game.roulette.is_none());
}

#[test]
fn player_damage_ends_the_run_and_progress_resets_on_next_run() {
    let (mut game, mut camera) = test_game();
    game.configure_mode(ProgramMode::Game, &mut camera);
    game.grant_player_experience(25);

    assert_eq!(game.progression.level(), 2);
    assert!(!game.damage_player(99.0));
    assert!(game.damage_player(1.0));
    assert_eq!(game.session.state(), RunState::PlayerDestroyed);

    game.configure_mode(ProgramMode::Game, &mut camera);
    assert_eq!(game.progression, PlayerProgression::new());
    assert_eq!(game.session.state(), RunState::Playing);
}

#[test]
fn evasive_loop_invulnerability_blocks_player_damage() {
    let (mut game, mut camera) = test_game();
    game.configure_mode(ProgramMode::Game, &mut camera);
    game.ship.speed_multiplier = GAME_FLIGHT_SPEED_MULTIPLIER;
    assert!(game.ship.try_start_evasive_loop());

    let health = game.progression.health();
    assert!(!game.damage_player(50.0));
    assert_eq!(game.progression.health(), health);

    game.ship.update_free(&ShipControls::default(), 0.34);
    assert!(!game.damage_player(50.0));
    assert_eq!(game.progression.health(), health - 50.0);
}

#[test]
fn run_enemy_contact_is_connected_to_player_health() {
    let (mut game, mut camera) = test_game();
    let input = InputState::new();
    let mut effect = WarpEffect::new();
    game.configure_mode(ProgramMode::Game, &mut camera);
    game.run_enemies
        .spawn_at(game.ship.position, game.progression.level());

    game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);

    assert_eq!(game.progression.health(), 90.0);
    assert_eq!(game.session.state(), RunState::Playing);
}

#[test]
fn elite_beam_reaches_a_still_player_after_its_warning() {
    let (mut game, mut camera) = test_game();
    let input = InputState::new();
    let mut effect = WarpEffect::new();
    game.configure_mode(ProgramMode::Game, &mut camera);
    game.ship.speed_multiplier = 0.0;
    let elite_position = game.ship.position + game.ship.forward() * 40.0;
    game.run_enemies
        .spawn_kind_at(elite_position, game.progression.level(), EnemyKind::Elite);
    let mut warned = false;
    for _ in 0..(60 * 4) {
        game.update(&input, 1.0 / 60.0, &mut camera, &mut effect);
        warned |= game.enemy_beams.active_count() > 0;
        game.ship.position = START_POSITION;
        game.ship.velocity = Vec3::default();
    }
    assert!(warned);
    assert!(game.progression.health() < game.progression.max_health());
}
