use gc_proyecto_3::engine::math::Vec3;
use gc_proyecto_3::game::player::{Ship, ShipControls};
use gc_proyecto_3::game::warp::{
    arrival_point, safe_distance, Destination, Warp, WarpEvent, WARP_TRAVEL_TIME,
};
use gc_proyecto_3::renderer::warp_effect::WarpEffect;

fn forward_thrust() -> ShipControls {
    ShipControls {
        thrust: 1.0,
        ..ShipControls::default()
    }
}

#[test]
fn ship_accelerates_and_slows_down_gradually() {
    let mut ship = Ship::new(Vec3::default(), Vec3::new(0.0, 0.0, -1.0));
    ship.update_free(&forward_thrust(), 1.0 / 60.0);
    let first = ship.speed();
    for _ in 0..60 {
        ship.update_free(&forward_thrust(), 1.0 / 60.0);
    }
    assert!(ship.speed() > first * 10.0);
    assert!(ship.position.z < 0.0);
    let cruising = ship.speed();
    ship.update_free(&ShipControls::default(), 1.0 / 60.0);
    assert!(ship.speed() < cruising && ship.speed() > cruising * 0.9);
}

#[test]
fn ship_motion_depends_on_time_not_frame_count() {
    let run = |steps: usize| {
        let mut ship = Ship::new(Vec3::default(), Vec3::new(0.0, 0.0, -1.0));
        for _ in 0..steps {
            ship.update_free(&forward_thrust(), 2.0 / steps as f32);
        }
        ship.position
    };
    assert!((run(60) - run(240)).length() < 0.3);
}

#[test]
fn orbit_mode_keeps_altitude_while_the_planet_moves() {
    let mut center = Vec3::new(30.0, 0.0, 0.0);
    let mut ship = Ship::new(center + Vec3::new(0.0, 0.0, 9.0), Vec3::new(1.0, 0.0, 0.0));
    ship.enter_orbit(0, center, 3.0);
    ship.update_orbit(
        &ShipControls::default(),
        center,
        Vec3::default(),
        3.0,
        3.0,
        1.0 / 60.0,
    );
    assert!(ship.orbit_transition() > 0.0 && ship.orbit_transition() < 1.0);
    for _ in 0..300 {
        let displacement = Vec3::new(0.0, 0.0, 0.05);
        center = center + displacement;
        ship.update_orbit(
            &ShipControls::default(),
            center,
            displacement,
            3.0,
            3.0,
            1.0 / 60.0,
        );
        assert!(((ship.position - center).length() - 9.0).abs() < 1e-2);
    }
    assert!(ship.is_orbiting());
    ship.leave_orbit();
    assert!(!ship.is_orbiting());
    assert!(ship.speed() > 0.0);
}

#[test]
fn orbit_controls_rotate_zoom_and_release_the_ship() {
    let center = Vec3::default();
    let radius = 3.0;
    let mut ship = Ship::new(Vec3::new(0.0, 2.0, 7.0), Vec3::new(1.0, 0.0, 0.0));
    ship.enter_orbit(0, center, radius);
    for _ in 0..120 {
        ship.update_orbit(
            &ShipControls::default(),
            center,
            Vec3::default(),
            radius,
            radius,
            1.0 / 60.0,
        );
    }

    let before_rotation = ship.position;
    for _ in 0..60 {
        ship.update_orbit(
            &ShipControls {
                yaw: 1.0,
                ..ShipControls::default()
            },
            center,
            Vec3::default(),
            radius,
            radius,
            1.0 / 60.0,
        );
    }
    assert!((ship.position - before_rotation).length() > 1.0);
    assert!((ship.position.y - 2.0).abs() < 1e-3);
    assert!(ship.up().dot(Vec3::new(0.0, 1.0, 0.0)) > 0.99);

    let before_zoom = ship.position.length();
    for _ in 0..30 {
        ship.update_orbit(
            &ShipControls {
                thrust: 1.0,
                ..ShipControls::default()
            },
            center,
            Vec3::default(),
            radius,
            radius,
            1.0 / 60.0,
        );
    }
    assert!(ship.position.length() < before_zoom);

    let mut released = false;
    for _ in 0..180 {
        released = ship.update_orbit(
            &ShipControls {
                thrust: -1.0,
                ..ShipControls::default()
            },
            center,
            Vec3::default(),
            radius,
            radius,
            1.0 / 60.0,
        );
        if released {
            break;
        }
    }
    assert!(released);
    assert!(!ship.is_orbiting());
    assert!(ship.velocity.dot(ship.position.normalize()) > 0.0);
}

#[test]
fn orbit_capture_requires_getting_close_to_the_planet() {
    assert!(Ship::can_orbit(8.0, 3.0));
    assert!(!Ship::can_orbit(9.0, 3.0));
}

#[test]
fn enlarged_body_releases_using_its_smaller_capture_radius() {
    let center = Vec3::default();
    let visual_radius = 18.0;
    let capture_radius = 12.0;
    let mut ship = Ship::new(Vec3::new(0.0, 0.0, 27.0), Vec3::new(1.0, 0.0, 0.0));
    ship.enter_orbit(0, center, visual_radius);

    let mut released = false;
    for _ in 0..90 {
        released = ship.update_orbit(
            &ShipControls {
                thrust: -1.0,
                ..ShipControls::default()
            },
            center,
            Vec3::default(),
            visual_radius,
            capture_radius,
            1.0 / 60.0,
        );
        if released {
            break;
        }
    }

    assert!(released);
    assert!(ship.position.length() < 40.0);
}

#[test]
fn warp_reaches_a_moving_target_at_a_safe_distance_and_faces_it() {
    let mut ship = Ship::new(Vec3::new(0.0, 0.0, 100.0), Vec3::new(0.0, 0.0, -1.0));
    let mut effect = WarpEffect::new();
    let mut warp = Warp::new();
    assert!(warp.engage(2, &mut ship, &mut effect));
    assert!(effect.is_active());
    assert!(!warp.engage(2, &mut ship, &mut effect));
    let step = 1.0 / 60.0;
    let mut time = 0.0;
    let mut arrived = None;
    let mut destination = Destination {
        center: Vec3::default(),
        radius: 3.0,
    };
    while arrived.is_none() && time < WARP_TRAVEL_TIME * 2.0 {
        time += step;
        destination.center = Vec3::new(time * 4.0, 0.0, 0.0);
        if let WarpEvent::Arrived(target) = warp.update(step, &mut ship, destination) {
            arrived = Some(target);
        }
    }
    assert_eq!(arrived, Some(2));
    let distance = (ship.position - destination.center).length();
    assert!((distance - safe_distance(3.0)).abs() < 1e-3);
    let facing = (destination.center - ship.position).normalize();
    assert!(ship.forward().dot(facing) > 0.99);
    assert!(!warp.is_ready());
}

#[test]
fn arrival_point_is_outside_the_body() {
    let destination = Destination {
        center: Vec3::new(5.0, 0.0, 0.0),
        radius: 4.0,
    };
    let point = arrival_point(Vec3::new(5.0, 0.0, 0.0), destination);
    assert!((point - destination.center).length() > destination.radius);
}
