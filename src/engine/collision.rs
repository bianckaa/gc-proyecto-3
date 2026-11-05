use super::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Collider {
    pub center: Vec3,
    pub radius: f32,
    pub flattening: f32,
}

impl Collider {
    pub fn sphere(center: Vec3, radius: f32) -> Self {
        Self {
            center,
            radius,
            flattening: 1.0,
        }
    }

    pub fn flattened(center: Vec3, radius: f32, flattening: f32) -> Self {
        Self {
            center,
            radius,
            flattening: flattening.clamp(0.05, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contact {
    pub position: Vec3,
    pub velocity: Vec3,
    pub normal: Vec3,
}

pub fn resolve_collision(
    position: Vec3,
    velocity: Vec3,
    radius: f32,
    collider: &Collider,
    margin: f32,
) -> Option<Contact> {
    let padding = radius + margin;
    let horizontal = collider.radius + padding;
    let vertical = collider.radius * collider.flattening + padding;
    let offset = position - collider.center;
    let scaled = Vec3::new(
        offset.x / horizontal,
        offset.y / vertical,
        offset.z / horizontal,
    );
    let distance = scaled.length();
    if distance >= 1.0 {
        return None;
    }
    let direction = if distance > f32::EPSILON {
        scaled * (1.0 / distance)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let surface = Vec3::new(
        direction.x * horizontal,
        direction.y * vertical,
        direction.z * horizontal,
    );
    let normal = Vec3::new(
        direction.x / horizontal,
        direction.y / vertical,
        direction.z / horizontal,
    )
    .normalize();
    let inward_speed = velocity.dot(normal);
    let velocity = if inward_speed < 0.0 {
        velocity - normal * inward_speed
    } else {
        velocity
    };
    Some(Contact {
        position: collider.center + surface,
        velocity,
        normal,
    })
}

pub fn resolve_all(
    mut position: Vec3,
    mut velocity: Vec3,
    radius: f32,
    colliders: &[Collider],
    margin: f32,
) -> (Vec3, Vec3, bool) {
    let mut collided = false;
    for collider in colliders {
        if let Some(contact) = resolve_collision(position, velocity, radius, collider, margin) {
            position = contact.position;
            velocity = contact.velocity;
            collided = true;
        }
    }
    (position, velocity, collided)
}
