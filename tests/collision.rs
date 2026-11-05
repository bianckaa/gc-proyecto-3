use gc_proyecto_3::engine::collision::{resolve_all, resolve_collision, Collider};
use gc_proyecto_3::engine::math::Vec3;

#[test]
fn ship_inside_a_planet_is_pushed_out_and_loses_inward_speed() {
    let planet = Collider::sphere(Vec3::default(), 5.0);
    let contact = resolve_collision(
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(-3.0, 2.0, 0.0),
        1.0,
        &planet,
        0.5,
    )
    .unwrap();
    assert!((contact.position.length() - 6.5).abs() < 1e-4);
    assert!(contact.velocity.x.abs() < 1e-4);
    assert!((contact.velocity.y - 2.0).abs() < 1e-4);
}

#[test]
fn ship_moving_away_keeps_its_speed() {
    let planet = Collider::sphere(Vec3::default(), 5.0);
    let contact = resolve_collision(
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0),
        1.0,
        &planet,
        0.0,
    )
    .unwrap();
    assert_eq!(contact.velocity, Vec3::new(0.0, 3.0, 0.0));
}

#[test]
fn far_objects_do_not_collide() {
    let planet = Collider::sphere(Vec3::default(), 5.0);
    assert!(
        resolve_collision(Vec3::new(9.0, 0.0, 0.0), Vec3::default(), 1.0, &planet, 0.5).is_none()
    );
}

#[test]
fn flattened_bodies_allow_flying_closer_above_them() {
    let disc = Collider::flattened(Vec3::default(), 6.0, 0.3);
    assert!(
        resolve_collision(Vec3::new(0.0, 3.0, 0.0), Vec3::default(), 0.5, &disc, 0.2).is_none()
    );
    assert!(
        resolve_collision(Vec3::new(5.0, 0.0, 0.0), Vec3::default(), 0.5, &disc, 0.2).is_some()
    );
}

#[test]
fn nothing_can_end_inside_any_body() {
    let bodies = [
        Collider::sphere(Vec3::default(), 7.0),
        Collider::sphere(Vec3::new(20.0, 0.0, 0.0), 2.0),
    ];
    for start in [Vec3::new(1.0, 1.0, 1.0), Vec3::new(19.5, 0.5, 0.0)] {
        let (position, _, collided) = resolve_all(start, Vec3::default(), 0.5, &bodies, 0.2);
        assert!(collided);
        for body in &bodies {
            assert!((position - body.center).length() >= body.radius + 0.5);
        }
    }
}
