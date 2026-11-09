use crate::engine::collision::Collider;
use crate::engine::math::Vec3;
use crate::world::generation::SeededRandom;
use std::collections::HashSet;

pub const SECTOR_SIZE: f32 = 300.0;
const ACTIVE_RADIUS: i32 = 5;
const PLANET_CHANCE: f32 = 0.30;
const SECTOR_MARGIN: f32 = 40.0;
const MIN_PLANET_SPACING: f32 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SectorCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SectorCoord {
    pub fn containing(position: Vec3) -> Self {
        Self {
            x: (position.x / SECTOR_SIZE).floor() as i32,
            y: (position.y / SECTOR_SIZE).floor() as i32,
            z: (position.z / SECTOR_SIZE).floor() as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetKind {
    Rocky,
    Gas,
    Core,
    Droid,
}

impl PlanetKind {
    pub const COUNT: usize = 4;

    pub fn index(self) -> usize {
        match self {
            Self::Rocky => 0,
            Self::Gas => 1,
            Self::Core => 2,
            Self::Droid => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunPlanet {
    pub id: u64,
    pub sector: SectorCoord,
    pub kind: PlanetKind,
    pub position: Vec3,
    pub radius: f32,
    pub health: f32,
    pub max_health: f32,
    pub rotation_phase: f32,
    pub spin_speed: f32,
    pub shape: Vec3,
    pub tint: Vec3,
}

impl RunPlanet {
    pub fn collider(self) -> Collider {
        Collider::sphere(self.position, self.radius)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DestroyedPlanet {
    pub id: u64,
    pub kind: PlanetKind,
    pub position: Vec3,
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveSector {
    coord: SectorCoord,
    planet: Option<RunPlanet>,
}

pub struct RunWorld {
    seed: u32,
    sectors: Vec<ActiveSector>,
    removed_planets: HashSet<u64>,
}

impl RunWorld {
    pub fn new() -> Self {
        Self {
            seed: 0,
            sectors: Vec::new(),
            removed_planets: HashSet::new(),
        }
    }

    pub fn reset(&mut self, seed: u32) {
        self.seed = seed;
        self.sectors.clear();
        self.removed_planets.clear();
    }

    pub fn clear(&mut self) {
        self.sectors.clear();
        self.removed_planets.clear();
    }

    pub fn update(&mut self, player_position: Vec3) {
        let center = SectorCoord::containing(player_position);
        self.sectors.retain(|sector| {
            (sector.coord.x - center.x).abs() <= ACTIVE_RADIUS
                && (sector.coord.y - center.y).abs() <= ACTIVE_RADIUS
                && (sector.coord.z - center.z).abs() <= ACTIVE_RADIUS
        });
        let mut existing: HashSet<_> = self.sectors.iter().map(|sector| sector.coord).collect();

        for x in (center.x - ACTIVE_RADIUS)..=(center.x + ACTIVE_RADIUS) {
            for y in (center.y - ACTIVE_RADIUS)..=(center.y + ACTIVE_RADIUS) {
                for z in (center.z - ACTIVE_RADIUS)..=(center.z + ACTIVE_RADIUS) {
                    let coord = SectorCoord { x, y, z };
                    if existing.contains(&coord) {
                        continue;
                    }
                    let planet = generate_planet(self.seed, coord)
                        .filter(|planet| !self.removed_planets.contains(&planet.id));
                    self.sectors.push(ActiveSector { coord, planet });
                    existing.insert(coord);
                }
            }
        }
        self.sectors.sort_by_key(|sector| sector.coord);
    }

    pub fn planets(&self) -> impl Iterator<Item = &RunPlanet> {
        self.sectors
            .iter()
            .filter_map(|sector| sector.planet.as_ref())
    }

    pub fn colliders(&self) -> impl Iterator<Item = Collider> + '_ {
        self.planets().map(|planet| planet.collider())
    }

    pub fn planet(&self, id: u64) -> Option<&RunPlanet> {
        self.planets().find(|planet| planet.id == id)
    }

    pub fn damage_planet(&mut self, id: u64, amount: f32) -> Option<DestroyedPlanet> {
        if !amount.is_finite() || amount <= 0.0 {
            return None;
        }
        let planet = self
            .sectors
            .iter_mut()
            .filter_map(|sector| sector.planet.as_mut())
            .find(|planet| planet.id == id)?;
        planet.health = (planet.health - amount).max(0.0);
        if planet.health > 0.0 {
            return None;
        }
        let destroyed = DestroyedPlanet {
            id,
            kind: planet.kind,
            position: planet.position,
            radius: planet.radius,
        };
        self.remove_planet(id);
        Some(destroyed)
    }

    pub fn remove_planet(&mut self, id: u64) -> Option<RunPlanet> {
        for sector in &mut self.sectors {
            if sector.planet.is_some_and(|planet| planet.id == id) {
                let planet = sector.planet.take().unwrap();
                self.removed_planets.insert(id);
                return Some(planet);
            }
        }
        None
    }

    pub fn active_sector_count(&self) -> usize {
        self.sectors.len()
    }
}

impl Default for RunWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
struct PlanetCandidate {
    planet: RunPlanet,
    priority: u32,
}

fn generate_planet(seed: u32, coord: SectorCoord) -> Option<RunPlanet> {
    let candidate = generate_candidate(seed, coord)?;
    for x in (coord.x - 1)..=(coord.x + 1) {
        for y in (coord.y - 1)..=(coord.y + 1) {
            for z in (coord.z - 1)..=(coord.z + 1) {
                let neighbor_coord = SectorCoord { x, y, z };
                if neighbor_coord == coord {
                    continue;
                }
                let Some(neighbor) = generate_candidate(seed, neighbor_coord) else {
                    continue;
                };
                let too_close = (candidate.planet.position - neighbor.planet.position).length()
                    < MIN_PLANET_SPACING;
                let neighbor_wins = neighbor.priority > candidate.priority
                    || (neighbor.priority == candidate.priority
                        && neighbor.planet.id < candidate.planet.id);
                if too_close && neighbor_wins {
                    return None;
                }
            }
        }
    }
    Some(candidate.planet)
}

fn generate_candidate(seed: u32, coord: SectorCoord) -> Option<PlanetCandidate> {
    let sector_seed = mix_sector_seed(seed, coord);
    let mut random = SeededRandom::new(sector_seed);
    if random.next() > PLANET_CHANCE {
        return None;
    }
    let kind = match random.next() {
        roll if roll < 0.30 => PlanetKind::Rocky,
        roll if roll < 0.55 => PlanetKind::Gas,
        roll if roll < 0.80 => PlanetKind::Core,
        _ => PlanetKind::Droid,
    };
    let (minimum_radius, maximum_radius, base_health) = match kind {
        PlanetKind::Rocky => (2.5, 4.0, 40.0),
        PlanetKind::Gas => (4.0, 6.0, 70.0),
        PlanetKind::Core => (3.0, 4.5, 100.0),
        PlanetKind::Droid => (3.2, 5.0, 90.0),
    };
    let radius = random.range(minimum_radius, maximum_radius);
    let sector_origin = Vec3::new(
        coord.x as f32 * SECTOR_SIZE,
        coord.y as f32 * SECTOR_SIZE,
        coord.z as f32 * SECTOR_SIZE,
    );
    let position = sector_origin
        + Vec3::new(
            random.range(SECTOR_MARGIN, SECTOR_SIZE - SECTOR_MARGIN),
            random.range(SECTOR_MARGIN, SECTOR_SIZE - SECTOR_MARGIN),
            random.range(SECTOR_MARGIN, SECTOR_SIZE - SECTOR_MARGIN),
        );
    let shape = match kind {
        PlanetKind::Rocky => Vec3::new(1.0, random.range(0.78, 1.0), random.range(0.82, 1.0)),
        PlanetKind::Gas => Vec3::new(1.0, random.range(0.72, 0.96), 1.0),
        PlanetKind::Core => Vec3::new(1.0, random.range(0.86, 1.0), random.range(0.88, 1.0)),
        PlanetKind::Droid => Vec3::new(1.0, random.range(0.75, 1.0), random.range(0.9, 1.0)),
    };
    let tint = match kind {
        PlanetKind::Rocky => Vec3::new(
            random.range(0.72, 1.25),
            random.range(0.65, 1.12),
            random.range(0.58, 1.05),
        ),
        PlanetKind::Gas => Vec3::new(
            random.range(0.7, 1.15),
            random.range(0.82, 1.25),
            random.range(0.9, 1.3),
        ),
        PlanetKind::Core | PlanetKind::Droid => Vec3::new(
            random.range(0.72, 1.2),
            random.range(0.72, 1.2),
            random.range(0.72, 1.2),
        ),
    };
    let planet = RunPlanet {
        id: ((sector_seed as u64) << 32) | sector_hash(coord) as u64,
        sector: coord,
        kind,
        position,
        radius,
        health: base_health,
        max_health: base_health,
        rotation_phase: random.range(0.0, std::f32::consts::TAU),
        spin_speed: random.range(-0.18, 0.18),
        shape,
        tint,
    };
    Some(PlanetCandidate {
        planet,
        priority: random.next().to_bits(),
    })
}

fn mix_sector_seed(seed: u32, coord: SectorCoord) -> u32 {
    let mut value = seed ^ sector_hash(coord);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^ (value >> 16)
}

fn sector_hash(coord: SectorCoord) -> u32 {
    (coord.x as u32)
        .wrapping_mul(73_856_093)
        .wrapping_add((coord.y as u32).wrapping_mul(19_349_663))
        .wrapping_add((coord.z as u32).wrapping_mul(83_492_791))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(seed: u32, player: Vec3) -> Vec<RunPlanet> {
        let mut world = RunWorld::new();
        world.reset(seed);
        world.update(player);
        world.planets().copied().collect()
    }

    #[test]
    fn same_seed_and_position_repeat_planets() {
        assert_eq!(snapshot(42, Vec3::default()), snapshot(42, Vec3::default()));
        assert_ne!(snapshot(42, Vec3::default()), snapshot(43, Vec3::default()));
    }

    #[test]
    fn active_window_streams_around_player() {
        let mut world = RunWorld::new();
        world.reset(7);
        world.update(Vec3::default());
        assert_eq!(world.active_sector_count(), 1331);

        let far_player = Vec3::new(SECTOR_SIZE * 10.0, 0.0, 0.0);
        world.update(far_player);
        assert_eq!(world.active_sector_count(), 1331);
        assert!(world
            .sectors
            .iter()
            .all(|sector| (sector.coord.x - 10).abs() <= ACTIVE_RADIUS));
    }

    #[test]
    fn planets_lose_health_and_are_removed_when_destroyed() {
        let mut world = RunWorld::new();
        world.reset(7);
        world.update(Vec3::default());
        let planet = *world.planets().next().expect("hay al menos un planeta");
        assert!(world.damage_planet(planet.id, planet.max_health * 0.5).is_none());
        assert!(world.planet(planet.id).unwrap().health < planet.max_health);
        let destroyed = world.damage_planet(planet.id, planet.max_health).unwrap();
        assert_eq!(destroyed.kind, planet.kind);
        assert!(world.planet(planet.id).is_none());
        world.update(Vec3::new(5_000.0, 0.0, 0.0));
        world.update(Vec3::default());
        assert!(world.planet(planet.id).is_none());
    }

    #[test]
    fn removed_planet_does_not_return_after_sector_reload() {
        let mut world = RunWorld::new();
        world.reset(12);
        world.update(Vec3::default());
        let planet = *world.planets().next().unwrap();
        world.remove_planet(planet.id);
        world.update(Vec3::new(SECTOR_SIZE * 10.0, 0.0, 0.0));
        world.update(Vec3::default());

        assert!(world.planets().all(|candidate| candidate.id != planet.id));
    }

    #[test]
    fn generated_planets_keep_minimum_spacing() {
        let planets = snapshot(91, Vec3::default());
        for (index, planet) in planets.iter().enumerate() {
            for other in &planets[(index + 1)..] {
                assert!((planet.position - other.position).length() >= MIN_PLANET_SPACING);
            }
        }
    }
}
