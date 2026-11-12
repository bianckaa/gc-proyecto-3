use crate::engine::collision::Collider;
use crate::engine::math::Vec3;
use crate::game::enemies::{EnemyKind, EnemySwarm};
use crate::game::enemy_beams::{BeamOwner, BeamRequest, EnemyBeams};
use crate::game::run_world::{PlanetKind, RunPlanet};

pub const ENGAGE_RANGE: f32 = 220.0;
const DROID_FIRE_INTERVAL: f32 = 3.0;
const CORE_FIRE_INTERVAL: f32 = 5.5;
const DROID_SPAWN_INTERVAL: f32 = 5.0;
const CORE_SPAWN_INTERVAL: f32 = 8.0;
const DROID_SUMMON_SIZE: usize = 1;
const CORE_SUMMON_SIZE: usize = 3;
const DROID_SUMMON_LIMIT: usize = 6;
const CORE_SUMMON_LIMIT: usize = 12;

#[derive(Debug, Clone)]
struct BossState {
    id: u64,
    kind: PlanetKind,
    fire_timer: f32,
    spawn_timer: f32,
    summoned: Vec<u64>,
}

impl BossState {
    fn new(planet: RunPlanet) -> Self {
        Self {
            id: planet.id,
            kind: planet.kind,
            fire_timer: if planet.kind == PlanetKind::Droid {
                1.5
            } else {
                2.5
            },
            spawn_timer: if planet.kind == PlanetKind::Droid {
                2.0
            } else {
                3.5
            },
            summoned: Vec::new(),
        }
    }

    fn fire_interval(&self) -> f32 {
        if self.kind == PlanetKind::Droid {
            DROID_FIRE_INTERVAL
        } else {
            CORE_FIRE_INTERVAL
        }
    }

    fn spawn_interval(&self) -> f32 {
        if self.kind == PlanetKind::Droid {
            DROID_SPAWN_INTERVAL
        } else {
            CORE_SPAWN_INTERVAL
        }
    }

    fn summon_size(&self) -> usize {
        if self.kind == PlanetKind::Droid {
            DROID_SUMMON_SIZE
        } else {
            CORE_SUMMON_SIZE
        }
    }

    fn summon_limit(&self) -> usize {
        if self.kind == PlanetKind::Droid {
            DROID_SUMMON_LIMIT
        } else {
            CORE_SUMMON_LIMIT
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BossEvents {
    pub shots: usize,
    pub summoned: usize,
}

#[derive(Default)]
pub struct BossPlanets {
    states: Vec<BossState>,
}

impl BossPlanets {
    pub fn clear(&mut self) {
        self.states.clear();
    }

    pub fn destroy(&mut self, id: u64, beams: &mut EnemyBeams) {
        self.states.retain(|state| state.id != id);
        beams.cancel_owner(BeamOwner::Structure(id));
    }

    pub fn update(
        &mut self,
        planets: &[RunPlanet],
        player_position: Vec3,
        player_level: u32,
        elapsed: f32,
        delta_time: f32,
        colliders: &[Collider],
        enemies: &mut EnemySwarm,
        beams: &mut EnemyBeams,
    ) -> BossEvents {
        let mut events = BossEvents::default();
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return events;
        }
        for planet in planets
            .iter()
            .copied()
            .filter(|planet| planet.kind.is_boss())
        {
            if !self.states.iter().any(|state| state.id == planet.id) {
                self.states.push(BossState::new(planet));
            }
        }

        for state in &mut self.states {
            let Some(planet) = planets.iter().find(|planet| planet.id == state.id) else {
                beams.cancel_owner(BeamOwner::Structure(state.id));
                continue;
            };
            state
                .summoned
                .retain(|id| enemies.position_of(*id).is_some());
            if (planet.position - player_position).length() > ENGAGE_RANGE {
                continue;
            }

            state.fire_timer -= delta_time;
            if state.fire_timer <= 0.0 {
                if beams.request(
                    BeamRequest {
                        owner: BeamOwner::Structure(state.id),
                        origin: planet.position,
                        target: player_position,
                    },
                    colliders,
                ) {
                    events.shots += 1;
                    state.fire_timer = state.fire_interval();
                }
            }

            state.spawn_timer -= delta_time;
            if state.spawn_timer > 0.0 || state.summoned.len() >= state.summon_limit() {
                continue;
            }
            let available = state.summon_limit() - state.summoned.len();
            let count = state.summon_size().min(available);
            for index in 0..count {
                let phase = elapsed * 0.7 + index as f32 * 2.399 + (state.id as u32) as f32;
                let direction =
                    Vec3::new(phase.cos(), (phase * 0.61).sin() * 0.45, phase.sin()).normalize();
                let position = planet.position + direction * (planet.radius + 4.0);
                let kind = if state.kind == PlanetKind::Core && index == 0 {
                    EnemyKind::Resistant
                } else {
                    EnemyKind::Common
                };
                let id = enemies.spawn_kind_at(position, player_level, kind);
                state.summoned.push(id);
                events.summoned += 1;
            }
            state.spawn_timer = state.spawn_interval();
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::run_world::SectorCoord;

    fn boss(id: u64, kind: PlanetKind) -> RunPlanet {
        RunPlanet {
            id,
            sector: SectorCoord { x: 0, y: 0, z: 0 },
            kind,
            position: Vec3::new(0.0, 0.0, -40.0),
            radius: 10.0,
            health: 1.0,
            max_health: 1.0,
            rotation_phase: 0.0,
            spin_speed: 0.0,
            shape: Vec3::new(1.0, 1.0, 1.0),
            tint: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    #[test]
    fn each_boss_respects_its_own_summon_limit() {
        let mut director = BossPlanets::default();
        let mut enemies = EnemySwarm::new();
        let mut beams = EnemyBeams::new();
        let planet = boss(7, PlanetKind::Droid);
        for step in 0..100 {
            director.update(
                &[planet],
                Vec3::default(),
                1,
                step as f32,
                1.0,
                &[],
                &mut enemies,
                &mut beams,
            );
        }
        assert_eq!(enemies.active_count(), DROID_SUMMON_LIMIT);
    }

    #[test]
    fn destroyed_boss_cancels_its_behavior() {
        let mut director = BossPlanets::default();
        let mut enemies = EnemySwarm::new();
        let mut beams = EnemyBeams::new();
        let planet = boss(9, PlanetKind::Core);
        director.update(
            &[planet],
            Vec3::default(),
            1,
            4.0,
            4.0,
            &[],
            &mut enemies,
            &mut beams,
        );
        director.destroy(planet.id, &mut beams);
        assert_eq!(director.states.len(), 0);
        assert_eq!(beams.active_count(), 0);
    }
}
