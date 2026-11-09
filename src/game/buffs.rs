use crate::game::weapons::Modifiers;

pub const BUFF_DURATION: f32 = 20.0;
pub const MAX_STACKS: u8 = 3;
pub const STACK_BONUS: f32 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffKind {
    ShipSpeed,
    Damage,
    FireRate,
    ProjectileSpeed,
    Shield,
}

impl BuffKind {
    pub const ALL: [BuffKind; 5] = [
        BuffKind::ShipSpeed,
        BuffKind::Damage,
        BuffKind::FireRate,
        BuffKind::ProjectileSpeed,
        BuffKind::Shield,
    ];

    pub fn index(self) -> usize {
        match self {
            BuffKind::ShipSpeed => 0,
            BuffKind::Damage => 1,
            BuffKind::FireRate => 2,
            BuffKind::ProjectileSpeed => 3,
            BuffKind::Shield => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BuffKind::ShipSpeed => "VELOCIDAD",
            BuffKind::Damage => "DANO",
            BuffKind::FireRate => "CADENCIA",
            BuffKind::ProjectileSpeed => "PROYECTILES",
            BuffKind::Shield => "ESCUDO",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ActiveBuffs {
    remaining: [f32; 5],
    stacks: [u8; 5],
}

impl ActiveBuffs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn grant(&mut self, kind: BuffKind) {
        let index = kind.index();
        self.stacks[index] = (self.stacks[index] + 1).min(MAX_STACKS);
        self.remaining[index] =
            (self.remaining[index] + BUFF_DURATION).min(BUFF_DURATION * MAX_STACKS as f32);
    }

    pub fn update(&mut self, delta_time: f32) {
        for index in 0..self.remaining.len() {
            if self.stacks[index] == 0 {
                continue;
            }
            self.remaining[index] -= delta_time;
            if self.remaining[index] <= 0.0 {
                self.remaining[index] = 0.0;
                self.stacks[index] = 0;
            }
        }
    }

    pub fn stacks(&self, kind: BuffKind) -> u8 {
        self.stacks[kind.index()]
    }

    pub fn remaining(&self, kind: BuffKind) -> f32 {
        self.remaining[kind.index()]
    }

    pub fn multiplier(&self, kind: BuffKind) -> f32 {
        1.0 + STACK_BONUS * self.stacks(kind) as f32
    }

    pub fn is_shielded(&self) -> bool {
        self.stacks(BuffKind::Shield) > 0
    }

    pub fn modifiers(&self) -> Modifiers {
        Modifiers {
            damage: self.multiplier(BuffKind::Damage),
            fire_rate: self.multiplier(BuffKind::FireRate),
            projectile_speed: self.multiplier(BuffKind::ProjectileSpeed),
        }
    }

    pub fn active(&self) -> impl Iterator<Item = (BuffKind, u8, f32)> + '_ {
        BuffKind::ALL
            .into_iter()
            .filter(|kind| self.stacks(*kind) > 0)
            .map(|kind| (kind, self.stacks(kind), self.remaining(kind)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffs_stack_extend_and_expire() {
        let mut buffs = ActiveBuffs::new();
        buffs.grant(BuffKind::Damage);
        buffs.grant(BuffKind::Damage);
        assert_eq!(buffs.stacks(BuffKind::Damage), 2);
        assert_eq!(buffs.multiplier(BuffKind::Damage), 1.5);
        assert_eq!(buffs.remaining(BuffKind::Damage), BUFF_DURATION * 2.0);
        for _ in 0..10 {
            buffs.grant(BuffKind::Damage);
        }
        assert_eq!(buffs.stacks(BuffKind::Damage), MAX_STACKS);
        buffs.update(BUFF_DURATION * MAX_STACKS as f32 + 1.0);
        assert_eq!(buffs.stacks(BuffKind::Damage), 0);
        assert_eq!(buffs.modifiers(), Modifiers::default());
    }

    #[test]
    fn shield_is_active_only_while_it_lasts() {
        let mut buffs = ActiveBuffs::new();
        buffs.grant(BuffKind::Shield);
        assert!(buffs.is_shielded());
        buffs.update(BUFF_DURATION + 0.1);
        assert!(!buffs.is_shielded());
    }
}
