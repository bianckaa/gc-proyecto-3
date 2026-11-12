use super::*;

impl EnemySwarm {
    pub fn elite_shots(
        &mut self,
        player_position: Vec3,
        delta_time: f32,
        mut try_fire: impl FnMut(u64, Vec3) -> bool,
    ) {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return;
        }
        for enemy in &mut self.enemies {
            if enemy.kind != EnemyKind::Elite || !enemy.is_materialized() {
                continue;
            }
            if (enemy.position - player_position).length() > ELITE_FIRE_RANGE {
                continue;
            }
            enemy.fire_timer -= delta_time;
            if enemy.fire_timer > 0.0 {
                continue;
            }
            enemy.fire_timer = if try_fire(enemy.id, enemy.position) {
                ELITE_FIRE_COOLDOWN + self.random.range(0.0, ELITE_FIRE_JITTER)
            } else {
                ELITE_FIRE_RETRY
            };
        }
    }

    pub fn damage(&mut self, id: u64, amount: f32) -> EnemyDamage {
        self.damage_with_knockback(id, amount, Vec3::default())
    }

    pub fn damage_with_knockback(&mut self, id: u64, amount: f32, impulse: Vec3) -> EnemyDamage {
        if !amount.is_finite() || amount <= 0.0 {
            return EnemyDamage::default();
        }
        let Some(index) = self.enemies.iter().position(|enemy| enemy.id == id) else {
            return EnemyDamage::default();
        };
        self.enemies[index].health = (self.enemies[index].health - amount).max(0.0);
        if impulse.x.is_finite() && impulse.y.is_finite() && impulse.z.is_finite() {
            self.enemies[index].knockback_velocity =
                self.enemies[index].knockback_velocity + impulse;
        }
        let destroyed = self.enemies[index].health <= 0.0;
        let kind = self.enemies[index].kind;
        if destroyed {
            self.enemies.swap_remove(index);
        }
        EnemyDamage { destroyed, kind }
    }

    pub fn update(
        &mut self,
        player_position: Vec3,
        _player_forward: Vec3,
        player_radius: f32,
        player_level: u32,
        obstacles: &[Collider],
        delta_time: f32,
    ) -> EnemyUpdate {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return EnemyUpdate::default();
        }

        self.contact_timer = (self.contact_timer - delta_time).max(0.0);

        for enemy in &mut self.enemies {
            enemy.sync(player_level, self.run_elapsed);
            enemy.arrival = (enemy.arrival + delta_time).min(ARRIVAL_DURATION);
            let to_player = player_position - enemy.position;
            let speed = pursuit_speed(to_player.length());
            let desired_heading = to_player.normalize();
            enemy.heading = turn_toward(
                enemy.heading,
                desired_heading,
                TURN_RATE_RADIANS * delta_time,
            );
            enemy.velocity = enemy.heading * speed + enemy.knockback_velocity;
            enemy.position = enemy.position + enemy.velocity * delta_time;
            enemy.knockback_velocity =
                enemy.knockback_velocity * (-KNOCKBACK_DAMPING * delta_time).exp();
        }
        self.enemies
            .retain(|enemy| (enemy.position - player_position).length() <= DESPAWN_DISTANCE);

        for _ in 0..SEPARATION_PASSES {
            self.separate_enemies();
            self.resolve_obstacles(obstacles);
        }

        let can_hit = self.contact_timer <= 0.0;
        let mut touching = false;
        let mut contact_damage: f32 = 0.0;
        let mut player_knockback = Vec3::default();
        if can_hit {
            for enemy in &mut self.enemies {
                if !enemy.is_materialized() {
                    continue;
                }
                let offset = player_position - enemy.position;
                let distance = offset.length();
                if distance > enemy.radius() + player_radius {
                    continue;
                }
                touching = true;
                contact_damage = contact_damage.max(enemy.kind.contact_damage());
                let away_from_enemy = if distance > f32::EPSILON {
                    offset * (1.0 / distance)
                } else {
                    -fallback_direction(0, enemy.id)
                };
                player_knockback = player_knockback + away_from_enemy;
                enemy.knockback_velocity =
                    enemy.knockback_velocity - away_from_enemy * ENEMY_CONTACT_KNOCKBACK;
            }
        }
        self.separate_from_player(player_position, player_radius);
        self.resolve_obstacles(obstacles);
        if touching {
            self.contact_timer = CONTACT_COOLDOWN;
            let direction = player_knockback.normalize();
            EnemyUpdate {
                contact_damage,
                player_knockback: direction * PLAYER_CONTACT_KNOCKBACK,
            }
        } else {
            EnemyUpdate::default()
        }
    }

    fn separate_enemies(&mut self) {
        for first in 0..self.enemies.len() {
            for second in (first + 1)..self.enemies.len() {
                let (left, right) = self.enemies.split_at_mut(second);
                let first_enemy = &mut left[first];
                let second_enemy = &mut right[0];
                let minimum_distance = first_enemy.radius() + second_enemy.radius();
                let offset = second_enemy.position - first_enemy.position;
                let distance = offset.length();
                if distance >= minimum_distance {
                    continue;
                }
                let direction = if distance > f32::EPSILON {
                    offset * (1.0 / distance)
                } else {
                    fallback_direction(first_enemy.id, second_enemy.id)
                };
                let correction = direction * ((minimum_distance - distance) * 0.5);
                first_enemy.position = first_enemy.position - correction;
                second_enemy.position = second_enemy.position + correction;
            }
        }
    }

    fn separate_from_player(&mut self, player_position: Vec3, player_radius: f32) {
        for enemy in &mut self.enemies {
            if !enemy.is_materialized() {
                continue;
            }
            let minimum_distance = enemy.radius() + player_radius;
            let offset = enemy.position - player_position;
            let distance = offset.length();
            if distance >= minimum_distance {
                continue;
            }
            let direction = if distance > f32::EPSILON {
                offset * (1.0 / distance)
            } else {
                fallback_direction(0, enemy.id)
            };
            enemy.position = player_position + direction * minimum_distance;
        }
    }

    fn resolve_obstacles(&mut self, obstacles: &[Collider]) {
        for enemy in &mut self.enemies {
            let (position, velocity, _) = resolve_all(
                enemy.position,
                enemy.velocity,
                enemy.radius(),
                obstacles,
                0.0,
            );
            enemy.position = position;
            enemy.velocity = velocity;
        }
    }
}
fn turn_toward(current: Vec3, target: Vec3, maximum_angle: f32) -> Vec3 {
    if target.length() <= f32::EPSILON {
        return current;
    }
    if current.length() <= f32::EPSILON {
        return target.normalize();
    }
    let current = current.normalize();
    let target = target.normalize();
    let angle = current.dot(target).clamp(-1.0, 1.0).acos();
    if angle <= maximum_angle || angle <= f32::EPSILON {
        return target;
    }
    let mut axis = current.cross(target);
    if axis.length() <= f32::EPSILON {
        let reference = if current.y.abs() < 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        axis = current.cross(reference);
    }
    rotate_around(current, axis.normalize(), maximum_angle.min(angle)).normalize()
}

fn fallback_direction(first_id: u64, second_id: u64) -> Vec3 {
    match first_id.wrapping_mul(31).wrapping_add(second_id) % 3 {
        0 => Vec3::new(1.0, 0.0, 0.0),
        1 => Vec3::new(0.0, 1.0, 0.0),
        _ => Vec3::new(0.0, 0.0, 1.0),
    }
}
