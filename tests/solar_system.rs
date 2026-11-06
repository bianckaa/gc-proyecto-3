use gc_proyecto_3::engine::math::Vec3;
use gc_proyecto_3::world::generation::{asteroid_belt, scatter_surface, surface_density};
use gc_proyecto_3::world::orbit::Orbit;
use gc_proyecto_3::world::{SolarSystem, SystemConfig};

#[test]
fn circular_orbit_keeps_its_radius_and_repeats_every_period() {
    let orbit = Orbit::circular(10.0, 0.5, 0.3).with_tilt(0.1, 0.4);
    for step in 0..20 {
        let time = step as f32 * 0.37;
        assert!((orbit.offset(time).length() - 10.0).abs() < 1e-3);
    }
    let start = orbit.offset(1.0);
    let later = orbit.offset(1.0 + orbit.period());
    assert!((start - later).length() < 1e-3);
}

#[test]
fn elliptic_orbit_changes_distance() {
    let orbit = Orbit::circular(10.0, 1.0, 0.0).with_eccentricity(0.3);
    let near = orbit.offset_at_angle(0.0).length();
    let far = orbit.offset_at_angle(std::f32::consts::PI).length();
    assert!(far > near + 1.0);
}

#[test]
fn standard_system_has_at_least_five_distinct_bodies() {
    let system = SolarSystem::new(SystemConfig::standard());
    assert!(system.body_count() >= 5);
    let mut shaders: Vec<_> = (0..system.body_count())
        .map(|index| format!("{:?}", system.body(index).material.shader))
        .collect();
    shaders.dedup();
    assert!(shaders.len() >= 5);
}

#[test]
fn menu_system_makes_bodies_larger_and_orbits_more_spacious() {
    let standard = SystemConfig::standard();
    let menu = SystemConfig::menu();

    for (standard_body, menu_body) in standard.bodies.iter().zip(&menu.bodies) {
        assert!(menu_body.radius > standard_body.radius);
        if standard_body.parent.is_some() {
            assert!(menu_body.orbit.radius > standard_body.orbit.radius);
        }
    }
    assert!(menu.belt.inner_radius > standard.belt.inner_radius);
    assert!(menu.belt.outer_radius > standard.belt.outer_radius);
}

#[test]
fn bodies_move_with_time_and_the_moon_follows_its_planet() {
    let mut system = SolarSystem::new(SystemConfig::standard());
    let moon = system.find("LUNA").unwrap();
    let planet = system.body(moon).parent.unwrap();
    let before = system.position(planet);
    for _ in 0..120 {
        system.update(1.0 / 60.0);
        let distance = (system.position(moon) - system.position(planet)).length();
        assert!((distance - system.body(moon).orbit.radius).abs() < 1e-2);
    }
    assert!((system.position(planet) - before).length() > 0.1);
    assert_eq!(system.sun_position(), Vec3::default());
}

#[test]
fn time_scale_can_change_live_without_jumps() {
    let mut system = SolarSystem::new(SystemConfig::standard());
    system.update(0.5);
    let before = system.position(3);
    system.config.time_scale = 3.0;
    system.update(1.0 / 60.0);
    assert!((system.position(3) - before).length() < 0.5);
    system.config.time_scale = 0.0;
    let frozen = system.position(3);
    system.update(1.0);
    assert_eq!(system.position(3), frozen);
}

#[test]
fn asteroid_belt_is_reproducible_and_inside_its_ring() {
    let config = SystemConfig::standard().belt;
    let first = asteroid_belt(&config);
    let second = asteroid_belt(&config);
    assert_eq!(first, second);
    assert_eq!(first.len(), config.count);
    for asteroid in &first {
        let position = asteroid.position(0.0);
        let ring = Vec3::new(position.x, 0.0, position.z).length();
        assert!(ring >= config.inner_radius - 1e-3 && ring <= config.outer_radius + 1e-3);
        assert!(position.y.abs() <= config.thickness * 0.5 + 1e-3);
    }
}

#[test]
fn surface_props_follow_the_noise_and_stand_on_the_normal() {
    let config = SystemConfig::standard()
        .bodies
        .iter()
        .find_map(|body| body.surface)
        .unwrap();
    let props = scatter_surface(&config);
    assert_eq!(props, scatter_surface(&config));
    assert!(!props.is_empty());
    for prop in &props {
        assert!((prop.direction.length() - 1.0).abs() < 1e-3);
        assert!(surface_density(&config, prop.direction) >= config.threshold);
        let up = prop.local_matrix(2.0) * Vec3::new(0.0, 1.0, 0.0).extend(0.0);
        assert!((up.truncate().normalize() - prop.direction).length() < 1e-3);
        let base = prop.local_matrix(2.0) * Vec3::default().extend(1.0);
        assert!((base.truncate().length() - 2.0).abs() < 1e-3);
    }
}
