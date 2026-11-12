#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponStats {
    pub level: u8,
    pub damage: f32,
    pub cooldown: f32,
    pub size: f32,
    pub knockback: f32,
    pub speed: f32,
    pub count: u32,
}

impl WeaponStats {
    pub fn modified(self, modifiers: Modifiers) -> Self {
        Self {
            level: self.level,
            damage: self.damage * modifiers.damage,
            cooldown: self.cooldown / modifiers.fire_rate.max(0.1),
            speed: self.speed * modifiers.projectile_speed,
            ..self
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "DANO {:.0}  RECARGA {:.2}S  TAMANO {:.1}",
            self.damage, self.cooldown, self.size
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Modifiers {
    pub damage: f32,
    pub fire_rate: f32,
    pub projectile_speed: f32,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            damage: 1.0,
            fire_rate: 1.0,
            projectile_speed: 1.0,
        }
    }
}
