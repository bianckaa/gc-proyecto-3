use crate::game::buffs::{ActiveBuffs, BuffKind, BUFF_DURATION};
use crate::game::progression::PlayerProgression;
use crate::game::run_world::PlanetKind;
use crate::game::weapons::{Inventory, WeaponKind};
use crate::world::generation::SeededRandom;

pub const ROCKY_EXPERIENCE: u32 = 60;
pub const FALLBACK_EXPERIENCE: u32 = 80;
pub const DROID_UNLOCK_CHANCE: f32 = 0.7;
pub const MAX_HEALTH_BONUS: f32 = 20.0;
pub const REPAIR_AMOUNT: f32 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reward {
    Experience(u32),
    Buff(BuffKind),
    Improve(WeaponKind),
    Unlock(WeaponKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeOption {
    Unlock(WeaponKind),
    Improve(WeaponKind),
    MaxHealth,
    Repair,
    Shield,
}

fn pick<T: Copy>(items: &[T], random: &mut SeededRandom) -> Option<T> {
    if items.is_empty() {
        return None;
    }
    let index = ((random.next() * items.len() as f32) as usize).min(items.len() - 1);
    Some(items[index])
}

pub fn planet_reward(kind: PlanetKind, inventory: &Inventory, random: &mut SeededRandom) -> Reward {
    let locked: Vec<_> = inventory.locked().collect();
    let upgradable: Vec<_> = inventory.upgradable().collect();
    match kind {
        PlanetKind::Rocky => Reward::Experience(ROCKY_EXPERIENCE),
        PlanetKind::Gas => Reward::Buff(pick(&BuffKind::ALL, random).unwrap_or(BuffKind::Damage)),
        PlanetKind::Core => pick(&upgradable, random)
            .map(Reward::Improve)
            .or_else(|| pick(&locked, random).map(Reward::Unlock))
            .unwrap_or(Reward::Experience(FALLBACK_EXPERIENCE)),
        PlanetKind::Droid => {
            let unlock_roll = random.next() < DROID_UNLOCK_CHANCE;
            let unlock = pick(&locked, random).map(Reward::Unlock);
            let improve = pick(&upgradable, random).map(Reward::Improve);
            if unlock_roll {
                unlock.or(improve)
            } else {
                improve.or(unlock)
            }
            .unwrap_or(Reward::Experience(FALLBACK_EXPERIENCE))
        }
    }
}

pub fn upgrade_options(
    inventory: &Inventory,
    random: &mut SeededRandom,
    count: usize,
) -> Vec<UpgradeOption> {
    let mut pool: Vec<UpgradeOption> = inventory
        .locked()
        .map(UpgradeOption::Unlock)
        .chain(inventory.upgradable().map(UpgradeOption::Improve))
        .collect();
    let mut options = Vec::with_capacity(count);
    while options.len() < count && !pool.is_empty() {
        let index = ((random.next() * pool.len() as f32) as usize).min(pool.len() - 1);
        options.push(pool.swap_remove(index));
    }
    for filler in [UpgradeOption::MaxHealth, UpgradeOption::Repair, UpgradeOption::Shield] {
        if options.len() < count {
            options.push(filler);
        }
    }
    options
}

pub fn describe(option: UpgradeOption, inventory: &Inventory) -> (String, String) {
    match option {
        UpgradeOption::Unlock(kind) => (
            format!("{} NUEVA", kind.name()),
            kind.stats(1).summary(),
        ),
        UpgradeOption::Improve(kind) => {
            let level = inventory.level(kind);
            let now = kind.stats(level);
            let next = kind.stats(level + 1);
            (
                format!("{} NV {} > {}", kind.name(), level, level + 1),
                format!(
                    "DANO {:.0} > {:.0}  RECARGA {:.2} > {:.2}",
                    now.damage, next.damage, now.cooldown, next.cooldown
                ),
            )
        }
        UpgradeOption::MaxHealth => (
            "BLINDAJE".to_owned(),
            format!("VIDA MAXIMA +{:.0}", MAX_HEALTH_BONUS),
        ),
        UpgradeOption::Repair => (
            "REPARACION".to_owned(),
            format!("RECUPERA {:.0} DE VIDA", REPAIR_AMOUNT),
        ),
        UpgradeOption::Shield => (
            "ESCUDO".to_owned(),
            format!("INVULNERABLE {:.0}S", BUFF_DURATION),
        ),
    }
}

pub fn apply_option(
    option: UpgradeOption,
    inventory: &mut Inventory,
    progression: &mut PlayerProgression,
    buffs: &mut ActiveBuffs,
) {
    match option {
        UpgradeOption::Unlock(kind) => {
            inventory.unlock(kind);
        }
        UpgradeOption::Improve(kind) => {
            inventory.upgrade(kind);
        }
        UpgradeOption::MaxHealth => progression.increase_max_health(MAX_HEALTH_BONUS, MAX_HEALTH_BONUS),
        UpgradeOption::Repair => progression.heal(REPAIR_AMOUNT),
        UpgradeOption::Shield => buffs.grant(BuffKind::Shield),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::weapons::MAX_WEAPON_LEVEL;

    fn maxed_inventory() -> Inventory {
        let mut inventory = Inventory::new();
        for kind in WeaponKind::ALL {
            inventory.unlock(kind);
            while inventory.upgrade(kind) {}
        }
        inventory
    }

    #[test]
    fn each_planet_kind_gives_its_reward_type() {
        let inventory = Inventory::new();
        let mut random = SeededRandom::new(3);
        assert!(matches!(
            planet_reward(PlanetKind::Rocky, &inventory, &mut random),
            Reward::Experience(_)
        ));
        assert!(matches!(
            planet_reward(PlanetKind::Gas, &inventory, &mut random),
            Reward::Buff(_)
        ));
        assert_eq!(
            planet_reward(PlanetKind::Core, &inventory, &mut random),
            Reward::Improve(WeaponKind::Normal)
        );
    }

    #[test]
    fn droid_prefers_unlocks_and_switches_to_upgrades_when_all_are_owned() {
        let inventory = Inventory::new();
        let unlocks = (0..200)
            .filter(|seed| {
                let mut random = SeededRandom::new(*seed);
                matches!(
                    planet_reward(PlanetKind::Droid, &inventory, &mut random),
                    Reward::Unlock(_)
                )
            })
            .count();
        assert!(unlocks > 100);

        let mut owned = Inventory::new();
        for kind in WeaponKind::ALL {
            owned.unlock(kind);
        }
        let mut random = SeededRandom::new(9);
        assert!(matches!(
            planet_reward(PlanetKind::Droid, &owned, &mut random),
            Reward::Improve(_)
        ));
    }

    #[test]
    fn maxed_inventory_never_offers_impossible_rewards() {
        let inventory = maxed_inventory();
        let mut random = SeededRandom::new(5);
        for kind in [PlanetKind::Core, PlanetKind::Droid] {
            assert_eq!(
                planet_reward(kind, &inventory, &mut random),
                Reward::Experience(FALLBACK_EXPERIENCE)
            );
        }
        let options = upgrade_options(&inventory, &mut random, 3);
        assert_eq!(options.len(), 3);
        assert!(options
            .iter()
            .all(|option| matches!(option, UpgradeOption::MaxHealth | UpgradeOption::Repair | UpgradeOption::Shield)));
        assert!(WeaponKind::ALL
            .iter()
            .all(|kind| inventory.level(*kind) == MAX_WEAPON_LEVEL));
    }

    #[test]
    fn three_distinct_options_are_generated_and_applied() {
        let mut inventory = Inventory::new();
        let mut progression = PlayerProgression::new();
        let mut buffs = ActiveBuffs::new();
        let mut random = SeededRandom::new(11);
        let options = upgrade_options(&inventory, &mut random, 3);
        assert_eq!(options.len(), 3);
        assert!(options[0] != options[1] && options[1] != options[2] && options[0] != options[2]);
        apply_option(UpgradeOption::Improve(WeaponKind::Normal), &mut inventory, &mut progression, &mut buffs);
        assert_eq!(inventory.level(WeaponKind::Normal), 2);
        apply_option(UpgradeOption::MaxHealth, &mut inventory, &mut progression, &mut buffs);
        assert!(progression.max_health() > 100.0);
    }
}
