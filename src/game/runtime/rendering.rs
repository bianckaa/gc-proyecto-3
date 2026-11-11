use super::super::*;

impl Game {
    pub fn populate(&self, scene: &mut Scene, assets: &AssetManager) {
        scene.clear();
        for index in 0..self.system.body_count() {
            let body = self.system.body(index);
            let state = self.system.state(index);
            let model = self.models.bodies[index];
            let unit_radius = match model {
                Model::Lod(handle) => assets.lod_bounding_radius(handle),
                Model::Mesh(handle) => assets.mesh_bounding_radius(handle),
            };
            scene.push(Entity {
                model,
                material: body.material,
                transform: Transform::new(state.position)
                    .with_rotation(state.rotation)
                    .with_uniform_scale(body.radius),
                radius: body.radius * unit_radius.max(1.0),
            });
        }

        for surface in &self.surfaces {
            let body = self.system.body(surface.body);
            let state = self.system.state(surface.body);
            let surface_radius = body.radius * (1.0 - surface.embed_depth);
            for prop in &surface.props {
                let position =
                    state.position + rotate_vector(state.rotation, prop.direction * surface_radius);
                let (model, material, scale) = match prop.kind {
                    PropKind::Crystal => (self.models.crystal, self.materials.crystal, prop.scale),
                    PropKind::Boulder => (
                        self.models.boulder,
                        self.materials.boulder,
                        prop.scale * 0.6,
                    ),
                };
                scene.push(Entity {
                    model,
                    material,
                    transform: Transform::new(position)
                        .with_rotation(state.rotation * prop.local_rotation())
                        .with_uniform_scale(scale),
                    radius: scale * 1.4,
                });
            }
        }

        let orbital_time = self.system.orbital_time();
        let system_origin = self.system.origin();
        for asteroid in &self.asteroids {
            scene.push(Entity {
                model: self.models.asteroid,
                material: self.materials.asteroid,
                transform: Transform::new(system_origin + asteroid.position(orbital_time))
                    .with_rotation(asteroid.rotation(self.time))
                    .with_scale(asteroid.scale),
                radius: asteroid.radius() * 1.4,
            });
        }

        for planet in self.run_world.planets() {
            let mut material = self.materials.run_planets[planet.kind.index()];
            material.base_color = Vec3::new(
                (material.base_color.x * planet.tint.x).clamp(0.0, 1.0),
                (material.base_color.y * planet.tint.y).clamp(0.0, 1.0),
                (material.base_color.z * planet.tint.z).clamp(0.0, 1.0),
            );
            if planet.kind == PlanetKind::Core {
                let pulse = 0.6 + 0.4 * (self.time * 2.5 + planet.rotation_phase).sin();
                material.emissive = material.emissive + CORE_GLOW * pulse;
            }
            scene.push(Entity {
                model: self.models.run_planets[planet.kind.index()],
                material,
                transform: Transform::new(planet.position)
                    .with_rotation(Mat4::rotation_y(
                        planet.rotation_phase + self.time * planet.spin_speed,
                    ))
                    .with_scale(Vec3::new(
                        planet.radius * planet.shape.x,
                        planet.radius * planet.shape.y,
                        planet.radius * planet.shape.z,
                    )),
                radius: planet.radius,
            });
        }

        for enemy in &self.menu_patrols {
            let host = self.system.position(enemy.host);
            let forward = enemy.orbit.direction_of_travel(self.time);
            let up = enemy.orbit.offset(self.time).normalize();
            scene.push(Entity {
                model: Model::Mesh(self.models.enemy),
                material: self.materials.enemy,
                transform: Transform::new(host + enemy.orbit.offset(self.time))
                    .with_rotation(
                        basis_rotation(forward, up) * Mat4::rotation_y(MODEL_FORWARD_CORRECTION),
                    )
                    .with_uniform_scale(enemy.scale),
                radius: enemy.scale,
            });
        }

        for enemy in self.run_enemies.enemies() {
            let forward = enemy.forward();
            let mut material = self.materials.enemy;
            match enemy.kind {
                EnemyKind::Common => {}
                EnemyKind::Resistant => material.base_color = RESISTANT_COLOR,
                EnemyKind::Elite => {
                    material.base_color = ELITE_COLOR;
                    material.emissive = ELITE_GLOW;
                }
            }
            let size = ENEMY_RENDER_SCALE * enemy.kind.scale();
            material.opacity = enemy.arrival_progress();
            material.outline = material.outline && enemy.is_materialized();
            scene.push(Entity {
                model: Model::Mesh(self.models.enemy),
                material,
                transform: Transform::new(enemy.position)
                    .with_rotation(
                        basis_rotation(forward, Vec3::new(0.0, 1.0, 0.0))
                            * Mat4::rotation_y(MODEL_FORWARD_CORRECTION),
                    )
                    .with_uniform_scale(size * enemy.arrival_scale()),
                radius: size,
            });
        }

        if !self.ship_visible() {
            return;
        }
        scene.push(Entity {
            model: Model::Mesh(self.models.ship),
            material: self.materials.ship,
            transform: Transform::new(self.ship.position)
                .with_rotation(self.ship.rotation() * Mat4::rotation_y(MODEL_FORWARD_CORRECTION))
                .with_uniform_scale(SHIP_RADIUS),
            radius: SHIP_RADIUS,
        });
    }

    pub fn queue_lines(&self, renderer: &mut Renderer) {
        for index in 0..self.system.body_count() {
            let Some(path) = self.system.orbit_path(index, ORBIT_SEGMENTS) else {
                continue;
            };
            let color = if index == self.destination {
                DESTINATION_COLOR
            } else {
                ORBIT_COLOR
            };
            for segment in path.windows(2).step_by(2) {
                renderer.draw_line_3d(segment[0], segment[1], color, false);
            }
        }
        if self.program_mode == ProgramMode::Game {
            self.weapons.draw(renderer, self.aim(), self.time);
        }
        for planet in self.run_world.planets() {
            draw_planet_signature(renderer, planet, self.time);
        }
        self.effects.draw(renderer);
        if self.program_mode == ProgramMode::Game {
            self.enemy_beams.draw(renderer, self.time);
            self.draw_enemy_bars(renderer);
        }
        if self.program_mode == ProgramMode::Game && self.buffs.is_shielded() && self.ship_visible()
        {
            let pulse = 1.6 + (self.time * 5.0).sin() * 0.12;
            let center = self.ship.position;
            let axes = [
                (self.ship.forward(), self.ship.right()),
                (self.ship.up(), self.ship.right()),
                (self.ship.forward(), self.ship.up()),
            ];
            for (u_axis, v_axis) in axes {
                let points: Vec<Vec3> = (0..=24)
                    .map(|index| {
                        let angle = index as f32 / 24.0 * std::f32::consts::TAU;
                        center + (u_axis * angle.cos() + v_axis * angle.sin()) * pulse
                    })
                    .collect();
                for segment in points.windows(2).step_by(2) {
                    renderer.draw_line_3d(segment[0], segment[1], SHIELD_COLOR, false);
                }
            }
        }
    }

    pub fn hud_info(&self) -> HudInfo {
        let flight_mode = if self.warp.is_traveling() {
            "WARP".to_owned()
        } else {
            match self.ship.mode {
                FlightMode::Free => "LIBRE".to_owned(),
                FlightMode::Orbit { body, .. } => format!("ORBITA {}", self.body_name(body)),
            }
        };
        let destination_center = self.system.position(self.destination);
        HudInfo {
            program_mode: match self.program_mode {
                ProgramMode::Menu => "MENU",
                ProgramMode::Game => "JUEGO",
            },
            objective: match self.program_mode {
                ProgramMode::Menu => Some("OBJETIVO: VUELA DENTRO DEL SOL PARA INICIAR"),
                ProgramMode::Game => match self.session.state() {
                    RunState::SolarCollapse => Some("COLAPSO SOLAR"),
                    RunState::PlayerDestroyed => Some("NAVE DESTRUIDA"),
                    _ => None,
                },
            },
            run_time_remaining: (self.program_mode == ProgramMode::Game)
                .then(|| self.session.remaining()),
            run_seed: if self.program_mode == ProgramMode::Game {
                self.session.seed()
            } else {
                self.next_run_seed
            },
            seed_input: self.seed_input.clone(),
            progression: (self.program_mode == ProgramMode::Game).then(|| hud::ProgressHudInfo {
                health: self.progression.health(),
                max_health: self.progression.max_health(),
                health_ratio: self.progression.health_ratio(),
                level: self.progression.level(),
                experience: self.progression.experience(),
                experience_required: self.progression.experience_required(),
                experience_ratio: self.progression.experience_ratio(),
            }),
            speed: self.ship.speed(),
            flight_mode,
            camera_mode: match self.camera_mode {
                CameraMode::Follow => "SEGUIR",
                CameraMode::Free => "LIBRE",
            },
            destination_number: self.destination,
            destination_name: self.body_name(self.destination),
            destination_distance: (destination_center - self.ship.position).length()
                - self.system.body(self.destination).radius,
            warp_status: match self.warp.state() {
                WarpState::Ready => "LISTO",
                WarpState::Traveling { .. } => "ACTIVO",
                WarpState::Cooldown { .. } => "RECARGA",
            },
            notice: self.notice.as_ref().map(|(message, _)| message.clone()),
            show_help: self.show_help,
            weapons: if self.program_mode == ProgramMode::Game {
                self.weapons
                    .inventory
                    .owned()
                    .map(|(kind, level)| format!("{} NV{}", kind.name(), level))
                    .collect()
            } else {
                Vec::new()
            },
            target: self.aimed_target(),
            buffs: self
                .buffs
                .active()
                .map(|(kind, stacks, remaining)| {
                    format!("{} X{} {:.0}S", kind.name(), stacks, remaining.ceil())
                })
                .collect(),
        }
    }

    pub fn light_position(&self) -> Vec3 {
        self.system.sun_position()
    }

    pub fn transition_flash(&self) -> f32 {
        match self.start_transition {
            StartTransition::Idle => 0.0,
            StartTransition::Flashing { elapsed, .. } => {
                let amount = if elapsed < START_FLASH_IN_DURATION {
                    elapsed / START_FLASH_IN_DURATION
                } else if elapsed < START_FLASH_IN_DURATION + START_FLASH_HOLD_DURATION {
                    1.0
                } else {
                    1.0 - (elapsed - START_FLASH_IN_DURATION - START_FLASH_HOLD_DURATION)
                        / START_FLASH_OUT_DURATION
                }
                .clamp(0.0, 1.0);
                amount * amount * (3.0 - 2.0 * amount)
            }
        }
    }
}

fn planet_ring(
    center: Vec3,
    u_axis: Vec3,
    v_axis: Vec3,
    radius: f32,
    segments: usize,
) -> Vec<Vec3> {
    (0..=segments)
        .map(|index| {
            let angle = index as f32 / segments as f32 * std::f32::consts::TAU;
            center + (u_axis * angle.cos() + v_axis * angle.sin()) * radius
        })
        .collect()
}

fn draw_planet_signature(renderer: &mut Renderer, planet: &run_world::RunPlanet, time: f32) {
    let tilt = planet.rotation_phase;
    let normal = Vec3::new(tilt.sin() * 0.35, 1.0, tilt.cos() * 0.35).normalize();
    let u_axis = normal.cross(Vec3::new(1.0, 0.0, 0.0)).normalize();
    let v_axis = normal.cross(u_axis);
    let center = planet.position;
    match planet.kind {
        PlanetKind::Gas => {
            for (scale, color) in [(1.7, GAS_RING_COLOR), (2.15, GAS_RING_DIM)] {
                let points = planet_ring(center, u_axis, v_axis, planet.radius * scale, 64);
                for segment in points.windows(2).step_by(2) {
                    renderer.draw_line_3d(segment[0], segment[1], color, false);
                }
            }
        }
        PlanetKind::Droid => {
            for index in 0..DROID_SATELLITES {
                let angle =
                    time * 0.8 + index as f32 * std::f32::consts::TAU / DROID_SATELLITES as f32;
                let orbit = planet.radius * 1.6;
                let position = center + (u_axis * angle.cos() + v_axis * angle.sin()) * orbit;
                let arm = planet.radius * 0.18;
                for axis in [u_axis, v_axis, normal] {
                    renderer.draw_line_3d(
                        position - axis * arm,
                        position + axis * arm,
                        DROID_COLOR,
                        false,
                    );
                }
            }
        }
        PlanetKind::Core => {
            let pulse = 1.25 + (time * 2.5 + tilt).sin() * 0.08;
            let points = planet_ring(center, u_axis, v_axis, planet.radius * pulse, 48);
            for segment in points.windows(2).skip(1).step_by(3) {
                renderer.draw_line_3d(segment[0], segment[1], CORE_HALO_COLOR, false);
            }
        }
        PlanetKind::Rocky => {
            let orbit = planet.radius * 1.35;
            for index in 0..ROCKY_DEBRIS {
                let angle = time * 0.3 + index as f32 * 0.9;
                let position = center + (u_axis * angle.cos() + v_axis * angle.sin()) * orbit;
                let nudge = normal * (planet.radius * 0.05);
                renderer.draw_line_3d(
                    position - nudge,
                    position + nudge,
                    ROCKY_DEBRIS_COLOR,
                    false,
                );
            }
        }
    }
}
