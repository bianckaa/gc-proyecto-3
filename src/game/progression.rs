pub const STARTING_MAX_HEALTH: f32 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExperienceGain {
    pub levels_gained: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerProgression {
    health: f32,
    max_health: f32,
    level: u32,
    experience: u32,
}

impl PlayerProgression {
    pub fn new() -> Self {
        Self {
            health: STARTING_MAX_HEALTH,
            max_health: STARTING_MAX_HEALTH,
            level: 1,
            experience: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn max_health(&self) -> f32 {
        self.max_health
    }

    pub fn health_ratio(&self) -> f32 {
        if self.max_health <= 0.0 {
            0.0
        } else {
            (self.health / self.max_health).clamp(0.0, 1.0)
        }
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn experience(&self) -> u32 {
        self.experience
    }

    pub fn experience_required(&self) -> u32 {
        experience_required_for(self.level)
    }

    pub fn experience_ratio(&self) -> f32 {
        self.experience as f32 / self.experience_required() as f32
    }

    pub fn is_destroyed(&self) -> bool {
        self.health <= 0.0
    }

    /// Returns true only when this damage destroys the player.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if !amount.is_finite() || amount <= 0.0 || self.is_destroyed() {
            return false;
        }
        self.health = (self.health - amount).max(0.0);
        self.is_destroyed()
    }

    pub fn heal(&mut self, amount: f32) {
        if amount.is_finite() && amount > 0.0 {
            self.health = (self.health + amount).min(self.max_health);
        }
    }

    pub fn increase_max_health(&mut self, amount: f32, heal_amount: f32) {
        if amount.is_finite() && amount > 0.0 {
            self.max_health += amount;
        }
        self.heal(heal_amount);
    }

    pub fn grant_experience(&mut self, amount: u32) -> ExperienceGain {
        self.experience = self.experience.saturating_add(amount);
        let mut levels_gained: u32 = 0;
        while self.experience >= self.experience_required() {
            self.experience -= self.experience_required();
            self.level = self.level.saturating_add(1);
            levels_gained = levels_gained.saturating_add(1);
        }
        ExperienceGain { levels_gained }
    }
}

impl Default for PlayerProgression {
    fn default() -> Self {
        Self::new()
    }
}

pub fn experience_required_for(level: u32) -> u32 {
    let completed = level.saturating_sub(1);
    20u32
        .saturating_add(completed.saturating_mul(12))
        .saturating_add(completed.saturating_mul(completed).saturating_mul(3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_healing_and_destruction_are_clamped() {
        let mut player = PlayerProgression::new();
        assert!(!player.take_damage(35.0));
        assert_eq!(player.health(), 65.0);
        player.heal(500.0);
        assert_eq!(player.health(), STARTING_MAX_HEALTH);
        assert!(player.take_damage(150.0));
        assert_eq!(player.health(), 0.0);
        assert!(!player.take_damage(1.0));
    }

    #[test]
    fn experience_can_gain_multiple_levels_and_keeps_remainder() {
        let mut player = PlayerProgression::new();
        let gain = player.grant_experience(60);

        assert_eq!(gain.levels_gained, 2);
        assert_eq!(player.level(), 3);
        assert_eq!(player.experience(), 5);
        assert_eq!(player.experience_required(), 56);
    }

    #[test]
    fn reset_restores_a_new_run() {
        let mut player = PlayerProgression::new();
        player.take_damage(40.0);
        player.grant_experience(25);
        player.increase_max_health(20.0, 10.0);
        player.reset();

        assert_eq!(player, PlayerProgression::new());
    }
}
