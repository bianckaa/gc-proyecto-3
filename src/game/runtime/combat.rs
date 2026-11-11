use super::super::*;

impl Game {
    fn open_roulette(&mut self) {
        let options = upgrade_options(&self.weapons.inventory, &mut self.reward_random, 3);
        let level = self.progression.level() + 1 - self.pending_upgrades.max(1);
        self.roulette = Some(Roulette::new(options, level));
        self.session.begin_upgrade_roulette();
    }

    pub(in crate::game) fn update_upgrades(&mut self, input: &InputState, delta_time: f32) {
        if self.program_mode != ProgramMode::Game {
            return;
        }
        if self.roulette.is_none()
            && self.pending_upgrades > 0
            && self.session.state() == RunState::Playing
        {
            self.open_roulette();
        }
        if self.session.state() != RunState::UpgradeRoulette {
            return;
        }
        let Some(roulette) = &mut self.roulette else {
            self.session.resume_playing();
            return;
        };
        if roulette.update(delta_time) {
            self.sounds.push(SoundEffect::RouletteTick);
        }
        if input.was_pressed(Key::A) || input.was_pressed(Key::Left) {
            roulette.move_selection(-1);
        }
        if input.was_pressed(Key::D) || input.was_pressed(Key::Right) {
            roulette.move_selection(1);
        }
        let mut choice = input
            .pressed_digit()
            .and_then(|index| roulette.choose(index));
        if choice.is_none() && input.was_pressed(Key::Enter) {
            choice = roulette.confirm();
        }
        if let Some(option) = choice {
            self.confirm_upgrade(option);
        }
    }

    pub(in crate::game) fn confirm_upgrade(&mut self, option: upgrades::UpgradeOption) {
        apply_option(
            option,
            &mut self.weapons.inventory,
            &mut self.progression,
            &mut self.buffs,
        );
        self.pending_upgrades = self.pending_upgrades.saturating_sub(1);
        self.roulette = None;
        if self.pending_upgrades > 0 {
            self.session.resume_playing();
            self.open_roulette();
        } else {
            self.session.resume_playing();
            self.notify("MEJORA APLICADA");
        }
    }

    pub(in crate::game) fn aim(&self) -> Aim {
        Aim {
            origin: self.ship.position,
            forward: self.ship.forward(),
            right: self.ship.right(),
            up: self.ship.up(),
        }
    }

    pub(in crate::game) fn weapon_modifiers(&self) -> Modifiers {
        self.buffs.modifiers()
    }

    pub fn take_sounds(&mut self) -> Vec<SoundEffect> {
        std::mem::take(&mut self.sounds)
    }

    pub fn run_stats(&self) -> RunStats {
        self.run_stats
    }

    pub fn weapon_levels(&self) -> Vec<(&'static str, u8)> {
        self.weapons
            .inventory
            .owned()
            .map(|(kind, level)| (kind.name(), level))
            .collect()
    }

    pub(in crate::game) fn handle_beam_events(&mut self, events: enemy_beams::BeamEvents) {
        if events.fired > 0 {
            self.sounds.push(SoundEffect::Laser);
        }
        for impact in events.impacts.iter().take(MAX_BEAM_IMPACT_EFFECTS) {
            self.effects.impact(*impact);
        }
        for (position, kind) in &events.enemies_destroyed {
            self.enemy_explosion(*position, *kind);
        }
        if events.player_damage > 0.0 {
            self.ship.velocity = self.ship.velocity + events.player_knockback;
            self.damage_player(events.player_damage);
        }
    }

    fn enemy_explosion(&mut self, position: Vec3, kind: EnemyKind) {
        match kind {
            EnemyKind::Common => self.effects.enemy_explosion(position),
            EnemyKind::Resistant => self.effects.tough_enemy_explosion(position, false),
            EnemyKind::Elite => self.effects.tough_enemy_explosion(position, true),
        }
    }

    pub(in crate::game) fn handle_combat_events(&mut self, events: CombatEvents) {
        for kind in &events.shots {
            self.sounds.push(match kind {
                weapons::WeaponKind::Laser => SoundEffect::Laser,
                weapons::WeaponKind::Bombs | weapons::WeaponKind::Boomerang => SoundEffect::Launch,
                weapons::WeaponKind::Aura => SoundEffect::Impact,
                _ => SoundEffect::Shot,
            });
        }
        if !events.impacts.is_empty() {
            self.sounds.push(SoundEffect::Impact);
        }
        if !events.enemies_destroyed.is_empty() {
            self.sounds.push(SoundEffect::EnemyDestroyed);
        }
        if !events.explosions.is_empty() {
            self.sounds.push(SoundEffect::BombExplosion);
        }
        if !events.planets_destroyed.is_empty() {
            self.sounds.push(SoundEffect::PlanetDestroyed);
        }
        for impact in events.impacts.iter().take(MAX_IMPACT_EFFECTS) {
            self.effects.impact(*impact);
        }
        for (position, impulse) in events.knockbacks.iter().take(MAX_IMPACT_EFFECTS) {
            self.effects.knockback(*position, *impulse);
        }
        for (position, amount) in &events.damage {
            self.push_damage_number(*position, *amount);
        }
        for (position, kind) in &events.enemies_destroyed {
            self.enemy_explosion(*position, *kind);
            self.run_stats.enemies_destroyed += 1;
            self.grant_player_experience(kind.experience());
        }
        for (center, radius) in &events.explosions {
            self.effects.bomb_explosion(*center, *radius);
        }
        for planet in &events.planets_destroyed {
            self.boss_planets.destroy(planet.id, &mut self.enemy_beams);
            self.effects
                .planet_explosion(planet.position, planet.radius, planet.kind);
            self.run_stats.planets_destroyed += 1;
            self.reward_planet(*planet);
        }
    }

    fn reward_planet(&mut self, planet: DestroyedPlanet) {
        let reward = planet_reward(
            planet.kind,
            &self.weapons.inventory,
            &mut self.reward_random,
        );
        let mut message = self.apply_reward(reward);
        if planet.kind.is_boss() {
            let bonus = planet_reward(
                planet.kind,
                &self.weapons.inventory,
                &mut self.reward_random,
            );
            let bonus_message = self.apply_reward(bonus);
            let experience = if planet.kind == PlanetKind::Core {
                150
            } else {
                100
            };
            self.grant_player_experience(experience);
            self.progression.heal(BOSS_HEAL_REWARD);
            message = format!(
                "RECOMPENSA JEFE: {}, {}, +{} XP, +{:.0} HP",
                message, bonus_message, experience, BOSS_HEAL_REWARD
            );
        }
        self.notify(format!(
            "PLANETA {} DESTRUIDO - {}",
            planet_name(planet.kind),
            message
        ));
    }

    fn apply_reward(&mut self, reward: Reward) -> String {
        match reward {
            Reward::Experience(amount) => {
                self.grant_player_experience(amount);
                format!("+{} XP", amount)
            }
            Reward::Buff(kind) => {
                self.buffs.grant(kind);
                format!(
                    "{} X{:.2} {:.0}S",
                    kind.name(),
                    self.buffs.multiplier(kind),
                    self.buffs.remaining(kind)
                )
            }
            Reward::Improve(kind) => {
                self.weapons.inventory.upgrade(kind);
                format!("{} NV {}", kind.name(), self.weapons.inventory.level(kind))
            }
            Reward::Unlock(kind) => {
                self.weapons.inventory.unlock(kind);
                format!("NUEVA ARMA {}", kind.name())
            }
        }
    }

    pub(in crate::game) fn refresh_colliders(&mut self) {
        self.colliders = self.system.colliders();
        let time = self.system.orbital_time();
        let origin = self.system.origin();
        self.colliders.extend(
            self.asteroids.iter().map(|asteroid| {
                Collider::sphere(origin + asteroid.position(time), asteroid.radius())
            }),
        );
        self.planet_collider_start = self.colliders.len();
        self.colliders.extend(self.run_world.colliders());
    }

    pub fn damage_player(&mut self, amount: f32) -> bool {
        if self.program_mode != ProgramMode::Game
            || self.session.state() != RunState::Playing
            || self.buffs.is_shielded()
            || self.ship.is_evasive_loop_invulnerable()
        {
            return false;
        }
        let destroyed = self.progression.take_damage(amount);
        self.damage_flash = DAMAGE_FLASH_DURATION;
        self.invulnerable_timer = INVULNERABILITY_DURATION;
        self.sounds.push(SoundEffect::Damage);
        if destroyed {
            self.session.destroy_player();
            self.effects.player_explosion(self.ship.position);
            self.ending_timer = 0.0;
            self.sounds.push(SoundEffect::PlanetDestroyed);
            self.ending_cause = Some("NAVE DESTRUIDA");
            self.ship.stop();
            self.notify("NAVE DESTRUIDA");
        }
        destroyed
    }

    pub fn grant_player_experience(&mut self, amount: u32) -> u32 {
        if self.program_mode != ProgramMode::Game || self.session.state() != RunState::Playing {
            return 0;
        }
        let gain = self.progression.grant_experience(amount);
        if gain.levels_gained > 0 {
            self.pending_upgrades += gain.levels_gained;
            self.sounds.push(SoundEffect::LevelUp);
            self.notify(format!("NIVEL {}", self.progression.level()));
        }
        gain.levels_gained
    }
}
