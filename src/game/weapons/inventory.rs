use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponKind {
    Normal,
    Gatling,
    Laser,
    Aura,
    Boomerang,
    Bombs,
}

impl WeaponKind {
    pub const ALL: [WeaponKind; 6] = [
        WeaponKind::Normal,
        WeaponKind::Gatling,
        WeaponKind::Laser,
        WeaponKind::Aura,
        WeaponKind::Boomerang,
        WeaponKind::Bombs,
    ];

    pub fn index(self) -> usize {
        match self {
            WeaponKind::Normal => 0,
            WeaponKind::Gatling => 1,
            WeaponKind::Laser => 2,
            WeaponKind::Aura => 3,
            WeaponKind::Boomerang => 4,
            WeaponKind::Bombs => 5,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Normal => "RAYO",
            WeaponKind::Gatling => "GATLING",
            WeaponKind::Laser => "LASER",
            WeaponKind::Aura => "AURA",
            WeaponKind::Boomerang => "BUMERAN",
            WeaponKind::Bombs => "BOMBAS",
        }
    }

    pub fn stats(self, level: u8) -> WeaponStats {
        let level = level.clamp(1, MAX_WEAPON_LEVEL);
        let step = level as f32 - 1.0;
        let mut stats = match self {
            WeaponKind::Normal => normal::stats(step),
            WeaponKind::Gatling => gatling::stats(step),
            WeaponKind::Laser => laser::stats(step),
            WeaponKind::Aura => aura::stats(step),
            WeaponKind::Boomerang => boomerang::stats(step),
            WeaponKind::Bombs => bombs::stats(step),
        };
        stats.level = level;
        stats
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inventory {
    levels: [u8; 6],
}

impl Inventory {
    pub fn new() -> Self {
        let mut levels = [0; 6];
        levels[WeaponKind::Normal.index()] = 1;
        Self { levels }
    }

    pub fn level(&self, kind: WeaponKind) -> u8 {
        self.levels[kind.index()]
    }

    pub fn owns(&self, kind: WeaponKind) -> bool {
        self.level(kind) > 0
    }

    pub fn is_maxed(&self, kind: WeaponKind) -> bool {
        self.level(kind) >= MAX_WEAPON_LEVEL
    }

    pub fn unlock(&mut self, kind: WeaponKind) -> bool {
        if self.owns(kind) {
            return self.upgrade(kind);
        }
        self.levels[kind.index()] = 1;
        true
    }

    pub fn upgrade(&mut self, kind: WeaponKind) -> bool {
        if !self.owns(kind) || self.is_maxed(kind) {
            return false;
        }
        self.levels[kind.index()] += 1;
        true
    }

    pub fn owned(&self) -> impl Iterator<Item = (WeaponKind, u8)> + '_ {
        WeaponKind::ALL
            .into_iter()
            .filter(|kind| self.owns(*kind))
            .map(|kind| (kind, self.level(kind)))
    }

    pub fn locked(&self) -> impl Iterator<Item = WeaponKind> + '_ {
        WeaponKind::ALL.into_iter().filter(|kind| !self.owns(*kind))
    }

    pub fn upgradable(&self) -> impl Iterator<Item = WeaponKind> + '_ {
        WeaponKind::ALL
            .into_iter()
            .filter(|kind| self.owns(*kind) && !self.is_maxed(*kind))
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
