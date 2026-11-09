use super::math::Vec3;

pub const SURFACE_GRAVITY_PER_RADIUS: f32 = 0.9;
pub const MAX_SURFACE_GRAVITY: f32 = 6.0;
pub const INFLUENCE_RADII: f32 = 10.0;
const MIN_DISTANCE: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GravityWell {
    pub center: Vec3,
    pub radius: f32,
}

impl GravityWell {
    pub const fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn surface_gravity(&self) -> f32 {
        (self.radius * SURFACE_GRAVITY_PER_RADIUS).min(MAX_SURFACE_GRAVITY)
    }

    pub fn acceleration_at(&self, position: Vec3) -> Vec3 {
        let offset = self.center - position;
        let distance = offset.length();
        if distance < MIN_DISTANCE || distance > self.radius * INFLUENCE_RADII {
            return Vec3::default();
        }
        let falloff = self.radius / distance.max(self.radius);
        offset * (self.surface_gravity() * falloff * falloff / distance)
    }
}

pub fn total_acceleration(position: Vec3, wells: &[GravityWell]) -> Vec3 {
    wells
        .iter()
        .fold(Vec3::default(), |total, well| total + well.acceleration_at(position))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_points_toward_the_body_and_weakens_with_distance() {
        let well = GravityWell::new(Vec3::default(), 2.0);
        let near = well.acceleration_at(Vec3::new(4.0, 0.0, 0.0));
        let far = well.acceleration_at(Vec3::new(8.0, 0.0, 0.0));
        assert!(near.x < 0.0 && near.y == 0.0 && near.z == 0.0);
        assert!((near.length() / far.length() - 4.0).abs() < 0.001);
    }

    #[test]
    fn surface_pull_matches_surface_gravity_and_is_capped() {
        let small = GravityWell::new(Vec3::default(), 2.0);
        let huge = GravityWell::new(Vec3::default(), 100.0);
        let at_surface = small.acceleration_at(Vec3::new(0.0, 2.0, 0.0)).length();
        assert!((at_surface - 2.0 * SURFACE_GRAVITY_PER_RADIUS).abs() < 0.001);
        assert!(huge.surface_gravity() <= MAX_SURFACE_GRAVITY);
    }

    #[test]
    fn bodies_beyond_their_influence_do_not_pull() {
        let well = GravityWell::new(Vec3::default(), 1.0);
        let outside = Vec3::new(INFLUENCE_RADII + 1.0, 0.0, 0.0);
        assert_eq!(well.acceleration_at(outside), Vec3::default());
        let wells = [well, GravityWell::new(Vec3::new(20.0, 0.0, 0.0), 1.0)];
        let between = total_acceleration(Vec3::new(10.0, 0.0, 0.0), &wells);
        assert!(between.length() < 0.0001);
    }
}
