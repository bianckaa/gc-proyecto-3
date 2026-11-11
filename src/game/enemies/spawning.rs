use super::*;

impl EnemySwarm {
    pub fn spawn_kind_at(&mut self, position: Vec3, player_level: u32, kind: EnemyKind) -> u64 {
        let id = self.allocate_id();
        self.enemies.push(RunEnemy::of_kind(
            id,
            position,
            player_level,
            kind,
            self.run_elapsed,
        ));
        id
    }

    pub fn spawn_kind_around(
        &mut self,
        motion: PlayerMotion,
        player_level: u32,
        kind: EnemyKind,
        obstacles: &[Collider],
        is_visible: impl Fn(Vec3) -> bool,
    ) -> u64 {
        let position = self.arrival_point(motion, obstacles, &is_visible);
        let id = self.allocate_id();
        let heading = (motion.position - position).normalize();
        self.enemies.push(RunEnemy::arriving(
            id,
            position,
            heading,
            player_level,
            kind,
            self.run_elapsed,
        ));
        id
    }

    pub fn spawn_at(&mut self, position: Vec3, player_level: u32) -> u64 {
        let id = self.allocate_id();
        self.enemies.push(RunEnemy::new(id, position, player_level));
        id
    }

    pub fn spawn_near_player(
        &mut self,
        player_position: Vec3,
        player_forward: Vec3,
        player_level: u32,
    ) -> u64 {
        let motion = PlayerMotion {
            position: player_position,
            velocity: Vec3::default(),
            forward: player_forward,
        };
        self.spawn_around(motion, player_level, &[], |_| false)
    }

    pub fn spawn_around(
        &mut self,
        motion: PlayerMotion,
        player_level: u32,
        obstacles: &[Collider],
        is_visible: impl Fn(Vec3) -> bool,
    ) -> u64 {
        self.spawn_kind_around(
            motion,
            player_level,
            EnemyKind::Common,
            obstacles,
            is_visible,
        )
    }

    pub fn recycle_stragglers(
        &mut self,
        motion: PlayerMotion,
        obstacles: &[Collider],
        is_visible: impl Fn(Vec3) -> bool,
        capacity: usize,
        delta_time: f32,
    ) -> Vec<Vec3> {
        let mut arrivals = Vec::new();
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return arrivals;
        }
        for enemy in &mut self.enemies {
            let far = (enemy.position - motion.position).length() > ENGAGE_DISTANCE;
            if far && !is_visible(enemy.position) {
                enemy.stray_time += delta_time;
            } else {
                enemy.stray_time = 0.0;
            }
        }

        let mut index = 0;
        while index < self.enemies.len() {
            if self.enemies[index].stray_time < RECYCLE_GRACE {
                index += 1;
                continue;
            }
            if self.engaged_count() >= capacity {
                self.enemies.swap_remove(index);
                continue;
            }
            let position = self.arrival_point(motion, obstacles, &is_visible);
            let enemy = &mut self.enemies[index];
            enemy.position = position;
            enemy.velocity = Vec3::default();
            enemy.heading = (motion.position - position).normalize();
            enemy.knockback_velocity = Vec3::default();
            enemy.arrival = 0.0;
            enemy.stray_time = 0.0;
            arrivals.push(position);
            index += 1;
        }
        arrivals
    }

    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }

    fn arrival_point(
        &mut self,
        motion: PlayerMotion,
        obstacles: &[Collider],
        is_visible: &impl Fn(Vec3) -> bool,
    ) -> Vec3 {
        let haste = (motion.velocity.length() / FAST_SHIP_SPEED).clamp(0.0, 1.0);
        let anchor = motion.position + motion.velocity * SPAWN_LOOKAHEAD;
        let behind = -motion.forward.normalize();
        let mut visible_fallback = None;
        let mut last_candidate = motion.position + behind * MIN_SPAWN_DISTANCE;
        for attempt in 0..SPAWN_ATTEMPTS {
            let spread = SPAWN_JITTER + haste * 2.0 + attempt as f32 * SPAWN_SPREAD_STEP;
            let jitter = self.random.unit_direction() * spread;
            let mut direction = (behind * (1.0 - haste) + jitter).normalize();
            if direction.length() <= f32::EPSILON {
                direction = behind;
            }
            let distance = self.random.range(MIN_SPAWN_DISTANCE, MAX_SPAWN_DISTANCE);
            let candidate = anchor + direction * distance;
            if (candidate - motion.position).length() < SPAWN_SAFE_RADIUS {
                continue;
            }
            last_candidate = candidate;
            if !is_clear_of_obstacles(candidate, obstacles) {
                continue;
            }
            if !is_visible(candidate) {
                return candidate;
            }
            visible_fallback.get_or_insert(candidate);
        }
        visible_fallback.unwrap_or(last_candidate)
    }
}

fn is_clear_of_obstacles(position: Vec3, obstacles: &[Collider]) -> bool {
    obstacles.iter().all(|obstacle| {
        (position - obstacle.center).length()
            > obstacle.radius + ENEMY_RADIUS * EnemyKind::Elite.scale() + SPAWN_CLEARANCE
    })
}
