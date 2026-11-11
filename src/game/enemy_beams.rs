use crate::engine::collision::Collider;
use crate::engine::math::Vec3;
use crate::game::enemies::{EnemyKind, EnemySwarm};
use crate::game::weapons::segment_sphere_hit;
use crate::renderer::framebuffer::Color;
use crate::renderer::Renderer;

pub const WARNING_DURATION: f32 = 0.9;
pub const FIRING_DURATION: f32 = 0.3;
pub const BEAM_RANGE: f32 = 140.0;
pub const BEAM_RADIUS: f32 = 0.7;
pub const BEAM_DAMAGE: f32 = 14.0;
pub const BEAM_KNOCKBACK: f32 = 9.0;
pub const GLOBAL_GAP: f32 = 0.45;
pub const MAX_ACTIVE_BEAMS: usize = 4;

const WARNING_COLOR: Color = Color::new(60, 180, 60);
const CORE_COLOR: Color = Color::new(220, 255, 220);
const GLOW_COLOR: Color = Color::new(70, 255, 90);
const WARNING_BLINK_START: f32 = 6.0;
const WARNING_BLINK_END: f32 = 22.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamOwner {
    Enemy(u64),
    Structure(u64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeamRequest {
    pub owner: BeamOwner,
    pub origin: Vec3,
    pub target: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BeamPhase {
    Warning,
    Firing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Beam {
    owner: BeamOwner,
    origin: Vec3,
    end: Vec3,
    phase: BeamPhase,
    elapsed: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BeamEvents {
    pub fired: usize,
    pub player_damage: f32,
    pub player_knockback: Vec3,
    pub enemies_destroyed: Vec<(Vec3, EnemyKind)>,
    pub impacts: Vec<Vec3>,
}

pub struct EnemyBeams {
    beams: Vec<Beam>,
    gap_timer: f32,
}

fn clipped_end(origin: Vec3, direction: Vec3, obstacles: &[Collider]) -> Vec3 {
    let end = origin + direction * BEAM_RANGE;
    let blocked = obstacles
        .iter()
        .filter(|collider| (origin - collider.center).length() > collider.radius)
        .filter_map(|collider| segment_sphere_hit(origin, end, collider.center, collider.radius))
        .min_by(f32::total_cmp);
    match blocked {
        Some(time) => origin + (end - origin) * time,
        None => end,
    }
}

impl EnemyBeams {
    pub fn new() -> Self {
        Self {
            beams: Vec::new(),
            gap_timer: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.beams.clear();
        self.gap_timer = 0.0;
    }

    pub fn active_count(&self) -> usize {
        self.beams.len()
    }

    pub fn request(&mut self, request: BeamRequest, obstacles: &[Collider]) -> bool {
        if self.gap_timer > 0.0 || self.beams.len() >= MAX_ACTIVE_BEAMS {
            return false;
        }
        let direction = (request.target - request.origin).normalize();
        if direction.length() <= f32::EPSILON {
            return false;
        }
        self.beams.push(Beam {
            owner: request.owner,
            origin: request.origin,
            end: clipped_end(request.origin, direction, obstacles),
            phase: BeamPhase::Warning,
            elapsed: 0.0,
        });
        self.gap_timer = GLOBAL_GAP;
        true
    }

    pub fn cancel_owner(&mut self, owner: BeamOwner) {
        self.beams.retain(|beam| beam.owner != owner);
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        player_position: Vec3,
        player_radius: f32,
        enemies: &mut EnemySwarm,
    ) -> BeamEvents {
        let mut events = BeamEvents::default();
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return events;
        }
        self.gap_timer = (self.gap_timer - delta_time).max(0.0);
        self.beams.retain(|beam| match beam.owner {
            BeamOwner::Enemy(id) => {
                beam.phase == BeamPhase::Firing || enemies.position_of(id).is_some()
            }
            BeamOwner::Structure(_) => true,
        });
        let mut firing_now = Vec::new();
        for beam in &mut self.beams {
            beam.elapsed += delta_time;
            if beam.phase == BeamPhase::Warning && beam.elapsed >= WARNING_DURATION {
                beam.phase = BeamPhase::Firing;
                beam.elapsed = 0.0;
                firing_now.push(*beam);
            }
        }
        self.beams
            .retain(|beam| beam.phase == BeamPhase::Warning || beam.elapsed < FIRING_DURATION);
        for beam in firing_now {
            events.fired += 1;
            self.resolve_hits(beam, player_position, player_radius, enemies, &mut events);
        }
        events
    }

    fn resolve_hits(
        &self,
        beam: Beam,
        player_position: Vec3,
        player_radius: f32,
        enemies: &mut EnemySwarm,
        events: &mut BeamEvents,
    ) {
        let direction = (beam.end - beam.origin).normalize();
        let impulse = direction * BEAM_KNOCKBACK;
        if let Some(time) = segment_sphere_hit(
            beam.origin,
            beam.end,
            player_position,
            player_radius + BEAM_RADIUS,
        ) {
            events.player_damage += BEAM_DAMAGE;
            events.player_knockback = events.player_knockback + impulse;
            events
                .impacts
                .push(beam.origin + (beam.end - beam.origin) * time);
        }
        let victims: Vec<(u64, Vec3, Vec3)> = enemies
            .enemies()
            .iter()
            .filter(|enemy| beam.owner != BeamOwner::Enemy(enemy.id))
            .filter_map(|enemy| {
                segment_sphere_hit(beam.origin, beam.end, enemy.position, enemy.radius() + BEAM_RADIUS)
                    .map(|time| (enemy.id, enemy.position, beam.origin + (beam.end - beam.origin) * time))
            })
            .collect();
        for (id, position, impact) in victims {
            events.impacts.push(impact);
            let result = enemies.damage_with_knockback(id, BEAM_DAMAGE, impulse);
            if result.destroyed {
                events.enemies_destroyed.push((position, result.kind));
            }
        }
    }

    pub fn draw(&self, renderer: &mut Renderer, time: f32) {
        for beam in &self.beams {
            match beam.phase {
                BeamPhase::Warning => {
                    let progress = beam.elapsed / WARNING_DURATION;
                    let rate = WARNING_BLINK_START + (WARNING_BLINK_END - WARNING_BLINK_START) * progress;
                    if (time * rate).sin() > -0.2 {
                        renderer.draw_line_3d(beam.origin, beam.end, WARNING_COLOR, true);
                    }
                }
                BeamPhase::Firing => {
                    let direction = (beam.end - beam.origin).normalize();
                    let helper = if direction.y.abs() < 0.9 {
                        Vec3::new(0.0, 1.0, 0.0)
                    } else {
                        Vec3::new(1.0, 0.0, 0.0)
                    };
                    let side = direction.cross(helper).normalize();
                    let up = side.cross(direction);
                    let fade = 1.0 - beam.elapsed / FIRING_DURATION;
                    let width = BEAM_RADIUS * (0.4 + 0.6 * fade);
                    renderer.draw_line_3d(beam.origin, beam.end, CORE_COLOR, false);
                    for offset in [side * width, -side * width, up * width, -up * width] {
                        renderer.draw_line_3d(beam.origin + offset, beam.end + offset, GLOW_COLOR, false);
                    }
                }
            }
        }
    }
}

impl Default for EnemyBeams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: f32 = 1.0 / 60.0;

    fn run(beams: &mut EnemyBeams, enemies: &mut EnemySwarm, player: Vec3, seconds: f32) -> BeamEvents {
        let mut total = BeamEvents::default();
        for _ in 0..(seconds / STEP) as usize {
            let events = beams.update(STEP, player, 0.9, enemies);
            total.fired += events.fired;
            total.player_damage += events.player_damage;
            total.enemies_destroyed.extend(events.enemies_destroyed);
        }
        total
    }

    fn shot(owner: BeamOwner, origin: Vec3, target: Vec3) -> BeamRequest {
        BeamRequest { owner, origin, target }
    }

    #[test]
    fn beam_warns_first_then_hurts_the_player() {
        let mut beams = EnemyBeams::new();
        let mut enemies = EnemySwarm::new();
        let player = Vec3::new(0.0, 0.0, -20.0);
        assert!(beams.request(shot(BeamOwner::Structure(1), Vec3::default(), player), &[]));
        let early = run(&mut beams, &mut enemies, player, WARNING_DURATION * 0.5);
        assert_eq!(early.player_damage, 0.0);
        let later = run(&mut beams, &mut enemies, player, WARNING_DURATION);
        assert_eq!(later.fired, 1);
        assert_eq!(later.player_damage, BEAM_DAMAGE);
    }

    #[test]
    fn dodging_during_the_warning_avoids_the_damage() {
        let mut beams = EnemyBeams::new();
        let mut enemies = EnemySwarm::new();
        beams.request(
            shot(BeamOwner::Structure(1), Vec3::default(), Vec3::new(0.0, 0.0, -20.0)),
            &[],
        );
        let events = run(&mut beams, &mut enemies, Vec3::new(8.0, 0.0, -20.0), 1.5);
        assert_eq!(events.fired, 1);
        assert_eq!(events.player_damage, 0.0);
    }

    #[test]
    fn beams_hurt_other_enemies_but_never_their_shooter() {
        let mut beams = EnemyBeams::new();
        let mut enemies = EnemySwarm::new();
        let shooter = enemies.spawn_kind_at(Vec3::default(), 1, EnemyKind::Elite);
        let victim = enemies.spawn_at(Vec3::new(0.0, 0.0, -15.0), 1);
        beams.request(
            shot(BeamOwner::Enemy(shooter), Vec3::default(), Vec3::new(0.0, 0.0, -40.0)),
            &[],
        );
        run(&mut beams, &mut enemies, Vec3::new(50.0, 0.0, 0.0), 1.5);
        let shooter_state = enemies.enemies().iter().find(|enemy| enemy.id == shooter).unwrap();
        let victim_state = enemies.enemies().iter().find(|enemy| enemy.id == victim).unwrap();
        assert_eq!(shooter_state.health, shooter_state.max_health);
        assert!(victim_state.health < victim_state.max_health);
    }

    #[test]
    fn obstacles_block_the_beam() {
        let mut beams = EnemyBeams::new();
        let mut enemies = EnemySwarm::new();
        let player = Vec3::new(0.0, 0.0, -30.0);
        let wall = [Collider::sphere(Vec3::new(0.0, 0.0, -15.0), 3.0)];
        beams.request(shot(BeamOwner::Structure(1), Vec3::default(), player), &wall);
        let events = run(&mut beams, &mut enemies, player, 1.5);
        assert_eq!(events.player_damage, 0.0);
    }

    #[test]
    fn cooldown_limits_simultaneous_beams() {
        let mut beams = EnemyBeams::new();
        let target = Vec3::new(0.0, 0.0, -20.0);
        assert!(beams.request(shot(BeamOwner::Structure(1), Vec3::default(), target), &[]));
        assert!(!beams.request(shot(BeamOwner::Structure(2), Vec3::default(), target), &[]));
        let mut enemies = EnemySwarm::new();
        run(&mut beams, &mut enemies, Vec3::new(50.0, 0.0, 0.0), GLOBAL_GAP + 0.05);
        assert!(beams.request(shot(BeamOwner::Structure(2), Vec3::default(), target), &[]));
    }

    #[test]
    fn dead_shooters_and_cancelled_owners_stop_firing() {
        let mut beams = EnemyBeams::new();
        let mut enemies = EnemySwarm::new();
        let shooter = enemies.spawn_kind_at(Vec3::default(), 1, EnemyKind::Elite);
        let player = Vec3::new(0.0, 0.0, -20.0);
        beams.request(shot(BeamOwner::Enemy(shooter), Vec3::default(), player), &[]);
        enemies.damage(shooter, 1_000_000.0);
        assert_eq!(run(&mut beams, &mut enemies, player, 1.5).fired, 0);
        beams.request(shot(BeamOwner::Structure(9), Vec3::default(), player), &[]);
        beams.cancel_owner(BeamOwner::Structure(9));
        assert_eq!(run(&mut beams, &mut enemies, player, 1.5).fired, 0);
    }
}
