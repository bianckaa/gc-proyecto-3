use super::super::*;

impl Game {
    pub fn black_hole(&self) -> (Vec3, f32) {
        let progress = self.collapse_progress();
        let intensity = ((progress - 0.15) / 0.6).clamp(0.0, 1.0);
        (
            self.system.sun_position(),
            intensity * intensity * (3.0 - 2.0 * intensity),
        )
    }

    pub fn damage_flash(&self) -> (Color, f32) {
        let amount = self.damage_flash / DAMAGE_FLASH_DURATION * DAMAGE_FLASH_STRENGTH;
        (DAMAGE_COLOR, amount.clamp(0.0, 1.0))
    }

    pub fn crosshair_screen_position(&self, camera: &Camera) -> Option<(i32, i32)> {
        if !self.ship_visible() {
            return None;
        }
        let target = self.ship.position + self.ship.forward() * CROSSHAIR_PREVIEW_DISTANCE;
        let clip = camera.view_projection_matrix() * target.extend(1.0);
        if clip.w <= 0.001 {
            return None;
        }
        let x = (clip.x / clip.w + 1.0) * 0.5 * FRAMEBUFFER_WIDTH as f32;
        let y = (1.0 - clip.y / clip.w) * 0.5 * FRAMEBUFFER_HEIGHT as f32;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let max_x = FRAMEBUFFER_WIDTH as i32 - 1 - CROSSHAIR_SCREEN_MARGIN;
        let max_y = FRAMEBUFFER_HEIGHT as i32 - 1 - CROSSHAIR_SCREEN_MARGIN;
        Some((
            (x.round() as i32).clamp(CROSSHAIR_SCREEN_MARGIN, max_x),
            (y.round() as i32).clamp(CROSSHAIR_SCREEN_MARGIN, max_y),
        ))
    }

    pub(in crate::game) fn ship_visible(&self) -> bool {
        if self.program_mode != ProgramMode::Game {
            return true;
        }
        if self.ship_absorbed || self.ending_cause == Some("NAVE DESTRUIDA") {
            return false;
        }
        self.invulnerable_timer <= 0.0 || (self.time * BLINK_RATE).sin() > 0.0
    }

    pub(in crate::game) fn aimed_target(&self) -> Option<(String, f32)> {
        if self.program_mode != ProgramMode::Game {
            return None;
        }
        let origin = self.ship.position;
        let forward = self.ship.forward();
        if let Some((_, boss)) = self
            .run_world
            .planets()
            .filter(|planet| planet.kind.is_boss())
            .filter_map(|planet| {
                let distance = (planet.position - origin).length();
                (distance <= BOSS_ENGAGE_RANGE).then_some((distance, planet))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
        {
            return Some((
                format!("JEFE {}", planet_name(boss.kind)),
                (boss.health / boss.max_health).clamp(0.0, 1.0),
            ));
        }
        let score = |position: Vec3| {
            let offset = position - origin;
            let distance = offset.length();
            let alignment = offset.dot(forward) / distance.max(f32::EPSILON);
            (distance <= TARGET_RANGE && alignment >= TARGET_CONE).then_some(distance)
        };
        let enemy = self.run_enemies.enemies().iter().filter_map(|enemy| {
            score(enemy.position).map(|distance| {
                (
                    distance,
                    "ENEMIGO".to_owned(),
                    enemy.health / enemy.max_health,
                )
            })
        });
        let planets = self.run_world.planets().filter_map(|planet| {
            score(planet.position).map(|distance| {
                (
                    distance,
                    format!("PLANETA {}", planet_name(planet.kind)),
                    planet.health / planet.max_health,
                )
            })
        });
        enemy
            .chain(planets)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, name, ratio)| (name, ratio.clamp(0.0, 1.0)))
    }

    pub fn draw_overlays(&self, renderer: &mut Renderer) -> Vec<TextLabel> {
        match (&self.roulette, &self.summary) {
            (Some(roulette), _) if self.session.state() == RunState::UpgradeRoulette => {
                roulette.draw(renderer, &self.weapons.inventory)
            }
            (_, Some(summary)) if self.session.state() == RunState::Summary => {
                draw_summary(renderer, summary, SUMMARY_AUTO_RETURN - self.ending_timer)
            }
            _ => Vec::new(),
        }
    }

    pub(in crate::game) fn push_damage_number(&mut self, position: Vec3, amount: f32) {
        if let Some(existing) = self.damage_numbers.iter_mut().find(|number| {
            (number.position - position).length() < DAMAGE_NUMBER_MERGE
                && number.life > DAMAGE_NUMBER_LIFE * 0.5
        }) {
            existing.amount += amount;
            existing.life = DAMAGE_NUMBER_LIFE;
            return;
        }
        if self.damage_numbers.len() >= MAX_DAMAGE_NUMBERS {
            self.damage_numbers.remove(0);
        }
        self.damage_numbers.push(DamageNumber {
            position,
            amount,
            life: DAMAGE_NUMBER_LIFE,
        });
    }

    pub(in crate::game) fn update_damage_numbers(&mut self, delta_time: f32) {
        let rise = self.ship.up() * (DAMAGE_NUMBER_RISE * delta_time);
        self.damage_numbers.retain_mut(|number| {
            number.life -= delta_time;
            number.position = number.position + rise;
            number.life > 0.0
        });
    }

    pub fn damage_labels(&self, camera: &Camera) -> Vec<TextLabel> {
        let view_projection = camera.view_projection_matrix();
        self.damage_numbers
            .iter()
            .filter_map(|number| {
                let clip = view_projection * number.position.extend(1.0);
                if clip.w <= 0.0 {
                    return None;
                }
                let x = (clip.x / clip.w + 1.0) * 0.5 * FRAMEBUFFER_WIDTH as f32;
                let y = (1.0 - clip.y / clip.w) * 0.5 * FRAMEBUFFER_HEIGHT as f32;
                Some(
                    TextLabel::new(
                        format!("{:.0}", number.amount),
                        x,
                        y,
                        DAMAGE_NUMBER_SIZE,
                        DAMAGE_NUMBER_COLOR,
                    )
                    .centered(),
                )
            })
            .collect()
    }

    pub(in crate::game) fn draw_enemy_bars(&self, renderer: &mut Renderer) {
        let up = self.ship.up();
        let right = self.ship.right();
        for enemy in self.run_enemies.enemies() {
            let color = match enemy.kind {
                EnemyKind::Common => continue,
                EnemyKind::Resistant => RESISTANT_BAR_COLOR,
                EnemyKind::Elite => ELITE_BAR_COLOR,
            };
            if !enemy.is_materialized() {
                continue;
            }
            let half = enemy.radius() * 1.1;
            let center = enemy.position + up * (enemy.radius() * 1.5 + 0.4);
            let start = center - right * half;
            let end = center + right * half;
            renderer.draw_line_3d(start, end, ENEMY_BAR_BACKGROUND, false);
            let filled = start + (end - start) * enemy.health_ratio();
            renderer.draw_line_3d(start, filled, color, false);
            if enemy.kind == EnemyKind::Elite {
                let lift = up * 0.12;
                renderer.draw_line_3d(start + lift, filled + lift, color, false);
            }
        }
    }
}
