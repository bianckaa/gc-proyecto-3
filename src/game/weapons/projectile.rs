use super::*;

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

pub(super) fn advance_projectiles(
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
