use super::*;

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
    pub fn nearest_enemy(&self, origin: Vec3) -> Option<Vec3> {
        self.nearest_enemies(origin, 1).into_iter().next()
    }

    pub fn nearest_enemies(&self, origin: Vec3, count: usize) -> Vec<Vec3> {
        let mut positions: Vec<_> = self
            .enemies
            .enemies()
            .iter()
            .map(|enemy| enemy.position)
            .collect();
        positions.sort_by(|a, b| (*a - origin).length().total_cmp(&(*b - origin).length()));
        positions.truncate(count);
        positions
    }

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

    pub fn enemies_within(&self, center: Vec3, radius: f32) -> Vec<(TargetId, Vec3)> {
        self.enemies
            .enemies()
            .iter()
            .filter(|enemy| (enemy.position - center).length() <= radius + enemy.radius())
            .map(|enemy| (TargetId::Enemy(enemy.id), enemy.position))
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
                    events
                        .knockbacks
                        .push((position.unwrap_or(impact), impulse));
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
