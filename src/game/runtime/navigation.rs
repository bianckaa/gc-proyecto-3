use super::super::*;

impl Game {
    pub(in crate::game) fn update_evasive_loop_input(&mut self, input: &InputState) {
        if !input.was_pressed(Key::R)
            || self.program_mode != ProgramMode::Game
            || self.session.state() != RunState::Playing
        {
            return;
        }
        if self.camera_mode != CameraMode::Follow || self.warp.is_traveling() {
            self.notify("ESQUIVA NO DISPONIBLE");
            return;
        }
        if self.ship.try_start_evasive_loop() {
            self.notify("LOOP EVASIVO");
        } else if self.ship.evasive_loop_cooldown() > 0.0 {
            self.notify(format!(
                "ESQUIVA RECARGANDO {:.1}S",
                self.ship.evasive_loop_cooldown()
            ));
        } else {
            self.notify("ESQUIVA NO DISPONIBLE");
        }
    }

    pub(in crate::game) fn update_menu_orbit_lockout(&mut self) {
        let Some(body) = self.menu_orbit_lockout else {
            return;
        };
        let distance = (self.system.position(body) - self.ship.position).length();
        let radius = self.system.body(body).orbit_capture_radius;
        if !Ship::can_orbit(distance, radius) {
            self.menu_orbit_lockout = None;
        }
    }

    pub(in crate::game) fn try_enter_menu_orbit(&mut self) {
        if self.program_mode != ProgramMode::Menu
            || self.ship.is_orbiting()
            || self.warp.is_traveling()
        {
            return;
        }
        let nearest = (1..self.system.body_count())
            .map(|body| {
                let distance = (self.system.position(body) - self.ship.position).length();
                (body, distance, self.system.body(body).orbit_capture_radius)
            })
            .filter(|&(body, distance, radius)| {
                self.menu_orbit_lockout != Some(body) && Ship::can_orbit(distance, radius)
            })
            .min_by(|a, b| (a.1 / a.2).total_cmp(&(b.1 / b.2)));
        if let Some((body, _, _)) = nearest {
            let radius = self.system.body(body).radius;
            self.ship
                .enter_orbit(body, self.system.position(body), radius);
            self.notify(format!(
                "ORBITA AUTOMATICA: {} - A/D GIRAR, W ACERCAR, S ALEJAR/SALIR",
                self.body_name(body)
            ));
        }
    }

    pub(in crate::game) fn try_start_game_transition(&mut self) {
        if self.program_mode != ProgramMode::Menu
            || !matches!(self.start_transition, StartTransition::Idle)
        {
            return;
        }
        let sun_distance = (self.ship.position - self.system.sun_position()).length();
        if sun_distance <= self.system.body(0).radius * SUN_ENTRY_DEPTH {
            self.ship.stop();
            self.notice = None;
            self.start_transition = StartTransition::Flashing {
                elapsed: 0.0,
                switched: false,
            };
        }
    }

    pub(in crate::game) fn update_start_transition(
        &mut self,
        delta_time: f32,
        camera: &mut Camera,
    ) {
        let StartTransition::Flashing {
            mut elapsed,
            mut switched,
        } = self.start_transition
        else {
            return;
        };
        elapsed += delta_time;
        if !switched && elapsed >= START_FLASH_IN_DURATION {
            self.configure_mode(ProgramMode::Game, camera);
            switched = true;
        }
        self.start_transition = if elapsed >= START_FLASH_DURATION {
            StartTransition::Idle
        } else {
            StartTransition::Flashing { elapsed, switched }
        };
    }

    pub(in crate::game) fn start_warp(&mut self, effect: &mut WarpEffect) {
        let target = self.destination;
        let distance = (self.system.position(target) - self.ship.position).length();
        if distance <= warp::safe_distance(self.system.body(target).radius) + 1.0 {
            self.notify(format!("YA ESTAS EN {}", self.body_name(target)));
            return;
        }
        if self.warp.engage(target, &mut self.ship, effect) {
            self.notify(format!("WARP HACIA {}", self.body_name(target)));
        } else if !self.warp.is_traveling() {
            self.notify("WARP RECARGANDO");
        }
    }

    fn gravity_wells(&self) -> Vec<GravityWell> {
        (0..self.system.body_count())
            .map(|index| {
                GravityWell::new(self.system.position(index), self.system.body(index).radius)
            })
            .collect()
    }

    pub(in crate::game) fn fly(&mut self, controls: &ShipControls, delta_time: f32) {
        match self.ship.mode {
            FlightMode::Free => {
                if self.gravity_enabled {
                    let pull = total_acceleration(self.ship.position, &self.gravity_wells());
                    self.ship.apply_acceleration(pull, delta_time);
                }
                self.ship.update_free(controls, delta_time);
            }
            FlightMode::Orbit { body, .. } => {
                let center = self.system.position(body);
                let displacement = self.system.displacement(body);
                let radius = self.system.body(body).radius;
                let capture_radius = self.system.body(body).orbit_capture_radius;
                if self.ship.update_orbit(
                    controls,
                    center,
                    displacement,
                    radius,
                    capture_radius,
                    delta_time,
                ) {
                    if self.program_mode == ProgramMode::Menu {
                        self.menu_orbit_lockout = Some(body);
                    }
                    self.notify("ORBITA LIBERADA - VUELO LIBRE");
                }
            }
        }
    }

    pub(in crate::game) fn follow_ship(&self, camera: &mut Camera, delta_time: f32) {
        let (forward, up, distance, height, look_ahead, smoothing) = match self.ship.orbit_body() {
            Some(body) => {
                let to_center = self.system.position(body) - self.ship.position;
                let orbit_forward = to_center.normalize();
                let transition = self.ship.orbit_transition();
                let eased = transition * transition * (3.0 - 2.0 * transition);
                let orbit_up = if orbit_forward.y.abs() > 0.92 {
                    Vec3::new(1.0, 0.0, 0.0)
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };
                (
                    (self.ship.forward() * (1.0 - eased) + orbit_forward * eased).normalize(),
                    (self.ship.up() * (1.0 - eased) + orbit_up * eased).normalize(),
                    self.follow_distance * (1.0 + (ORBIT_CAMERA_DISTANCE_FACTOR - 1.0) * eased),
                    self.follow_distance
                        * (FOLLOW_HEIGHT_RATIO * (1.0 - eased) + ORBIT_CAMERA_HEIGHT_RATIO * eased),
                    FOLLOW_LOOK_AHEAD + (to_center.length() - FOLLOW_LOOK_AHEAD) * eased,
                    FOLLOW_SMOOTHING + (ORBIT_CAMERA_SMOOTHING - FOLLOW_SMOOTHING) * eased,
                )
            }
            None if self.ship.is_evasive_looping() => (
                self.ship.evasive_camera_forward(),
                self.ship.evasive_camera_up(),
                self.follow_distance,
                self.follow_distance * FOLLOW_HEIGHT_RATIO,
                FOLLOW_LOOK_AHEAD,
                EVASIVE_LOOP_CAMERA_SMOOTHING,
            ),
            None => (
                self.ship.forward(),
                self.ship.up(),
                self.follow_distance,
                self.follow_distance * FOLLOW_HEIGHT_RATIO,
                FOLLOW_LOOK_AHEAD,
                FOLLOW_SMOOTHING,
            ),
        };
        camera.follow(
            self.ship.position,
            forward,
            up,
            distance,
            height,
            look_ahead,
            smoothing,
            delta_time,
        );
    }

    pub(in crate::game) fn fly_free_camera(
        &self,
        camera: &mut Camera,
        controls: &ShipControls,
        delta_time: f32,
    ) {
        camera.rotate(
            controls.yaw * FREE_CAMERA_YAW_RATE * delta_time,
            controls.pitch * FREE_CAMERA_PITCH_RATE * delta_time,
            controls.roll * FREE_CAMERA_ROLL_RATE * delta_time,
        );
        let speed = if controls.boost {
            FREE_CAMERA_BOOST
        } else {
            FREE_CAMERA_SPEED
        } * delta_time;
        camera.move_local(controls.thrust * speed, 0.0, controls.vertical * speed);
    }
}
