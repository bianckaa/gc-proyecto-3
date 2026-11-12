use crate::engine::math::{Mat4, Vec3};
use crate::engine::transform::{basis_rotation, exponential_blend, rotate_around};
use std::f32::consts::TAU;

pub const SHIP_RADIUS: f32 = 0.9;
const THRUST: f32 = 16.0;
const BOOST_THRUST: f32 = 42.0;
const VERTICAL_THRUST: f32 = 10.0;
const MAX_SPEED: f32 = 14.0;
const MAX_BOOST_SPEED: f32 = 36.0;
const DRAG: f32 = 0.55;
const SPEED_LIMIT_RESPONSE: f32 = 3.0;
const YAW_RATE: f32 = 1.4;
const PITCH_RATE: f32 = 1.1;
const ROLL_RATE: f32 = 2.0;
const TURN_RESPONSE: f32 = 4.0;
const MAX_BANK: f32 = 0.55;
const BANK_RESPONSE: f32 = 5.0;
const ORBIT_ANGULAR_RATE: f32 = 0.72;
const ORBIT_ZOOM_RATE: f32 = 9.0;
const ORBIT_RELEASE_SPEED: f32 = 5.0;
const ORBIT_MIN_ALTITUDE_FACTOR: f32 = 1.45;
const ORBIT_EXIT_ALTITUDE_FACTOR: f32 = 2.8;
const ORBIT_EXIT_MARGIN: f32 = 2.0;
const ORBIT_GRAB_RADIUS_FACTOR: f32 = 2.35;
const ORBIT_GRAB_MARGIN: f32 = 1.5;
const ORBIT_ALIGN_RESPONSE: f32 = 3.2;
const ORBIT_ENTRY_RESPONSE: f32 = 2.8;
const EVASIVE_LOOP_DURATION: f32 = 0.9;
const EVASIVE_LOOP_INVULNERABILITY: f32 = 0.32;
const EVASIVE_LOOP_COOLDOWN: f32 = 3.0;
const EVASIVE_LOOP_SPEED: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShipControls {
    pub thrust: f32,
    pub vertical: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub boost: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlightMode {
    Free,
    Orbit {
        body: usize,
        altitude: f32,
        height: f32,
        transition: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ship {
    pub position: Vec3,
    pub velocity: Vec3,
    pub speed_multiplier: f32,
    forward: Vec3,
    up: Vec3,
    angular_velocity: Vec3,
    bank: f32,
    evasive_loop_remaining: f32,
    evasive_loop_cooldown: f32,
    evasive_loop_forward: Vec3,
    evasive_loop_up: Vec3,
    pub mode: FlightMode,
}

impl Ship {
    pub fn new(position: Vec3, forward: Vec3) -> Self {
        let rotation = basis_rotation(forward, Vec3::new(0.0, 1.0, 0.0));
        Self {
            position,
            velocity: Vec3::default(),
            speed_multiplier: 1.0,
            forward: forward.normalize(),
            up: Vec3::new(
                rotation.rows[0][1],
                rotation.rows[1][1],
                rotation.rows[2][1],
            ),
            angular_velocity: Vec3::default(),
            bank: 0.0,
            evasive_loop_remaining: 0.0,
            evasive_loop_cooldown: 0.0,
            evasive_loop_forward: Vec3::default(),
            evasive_loop_up: Vec3::default(),
            mode: FlightMode::Free,
        }
    }

    pub fn forward(&self) -> Vec3 {
        self.forward
    }

    pub fn up(&self) -> Vec3 {
        self.up
    }

    pub fn right(&self) -> Vec3 {
        self.forward.cross(self.up).normalize()
    }

    pub fn speed(&self) -> f32 {
        self.velocity.length()
    }

    pub fn is_evasive_looping(&self) -> bool {
        self.evasive_loop_remaining > 0.0
    }

    pub fn evasive_loop_cooldown(&self) -> f32 {
        self.evasive_loop_cooldown
    }

    pub fn evasive_camera_forward(&self) -> Vec3 {
        if self.is_evasive_looping() {
            self.evasive_loop_forward
        } else {
            self.forward
        }
    }

    pub fn evasive_camera_up(&self) -> Vec3 {
        if self.is_evasive_looping() {
            self.evasive_loop_up
        } else {
            self.up
        }
    }

    pub fn is_evasive_loop_invulnerable(&self) -> bool {
        self.is_evasive_looping()
            && self.evasive_loop_remaining > EVASIVE_LOOP_DURATION - EVASIVE_LOOP_INVULNERABILITY
    }

    pub fn try_start_evasive_loop(&mut self) -> bool {
        if self.mode != FlightMode::Free
            || self.is_evasive_looping()
            || self.evasive_loop_cooldown > 0.0
        {
            return false;
        }
        self.evasive_loop_remaining = EVASIVE_LOOP_DURATION;
        self.evasive_loop_cooldown = EVASIVE_LOOP_COOLDOWN;
        self.evasive_loop_forward = self.forward;
        self.evasive_loop_up = self.up;
        self.angular_velocity = Vec3::default();
        self.bank = 0.0;
        self.velocity = self.forward * (EVASIVE_LOOP_SPEED * self.speed_multiplier);
        true
    }

    pub fn is_orbiting(&self) -> bool {
        matches!(self.mode, FlightMode::Orbit { .. })
    }

    pub fn orbit_body(&self) -> Option<usize> {
        match self.mode {
            FlightMode::Orbit { body, .. } => Some(body),
            FlightMode::Free => None,
        }
    }

    pub fn orbit_transition(&self) -> f32 {
        match self.mode {
            FlightMode::Orbit { transition, .. } => transition,
            FlightMode::Free => 0.0,
        }
    }

    pub fn set_orientation(&mut self, forward: Vec3, up: Vec3) {
        let forward = forward.normalize();
        let right = forward.cross(up).normalize();
        if right.length() < 0.5 {
            return;
        }
        self.forward = forward;
        self.up = right.cross(forward);
    }

    pub fn look_toward(&mut self, target: Vec3) {
        let direction = target - self.position;
        if direction.length() > f32::EPSILON {
            self.set_orientation(direction, self.up);
        }
    }

    pub fn stop(&mut self) {
        self.velocity = Vec3::default();
        self.angular_velocity = Vec3::default();
    }

    pub fn rotation(&self) -> Mat4 {
        basis_rotation(self.forward, self.up) * Mat4::rotation_z(self.bank)
    }

    fn steer(&mut self, controls: &ShipControls, delta_time: f32) {
        let target = Vec3::new(
            controls.yaw * YAW_RATE,
            controls.pitch * PITCH_RATE,
            controls.roll * ROLL_RATE,
        );
        let blend = exponential_blend(TURN_RESPONSE, delta_time);
        self.angular_velocity = self.angular_velocity + (target - self.angular_velocity) * blend;
        let rates = self.angular_velocity * delta_time;
        let right = self.right();
        let forward = rotate_around(self.forward, self.up, rates.x);
        let forward = rotate_around(forward, right, rates.y);
        let up = rotate_around(self.up, right, rates.y);
        let up = rotate_around(up, forward, rates.z);
        self.set_orientation(forward, up);
        let target_bank = controls.yaw * MAX_BANK;
        self.bank += (target_bank - self.bank) * exponential_blend(BANK_RESPONSE, delta_time);
    }

    pub fn apply_acceleration(&mut self, acceleration: Vec3, delta_time: f32) {
        self.velocity = self.velocity + acceleration * delta_time;
    }

    pub fn update_free(&mut self, controls: &ShipControls, delta_time: f32) {
        self.evasive_loop_cooldown = (self.evasive_loop_cooldown - delta_time).max(0.0);
        if self.is_evasive_looping() {
            self.update_evasive_loop(delta_time);
            return;
        }
        self.steer(controls, delta_time);
        let thrust = if controls.boost { BOOST_THRUST } else { THRUST } * self.speed_multiplier;
        let acceleration = self.forward * (controls.thrust * thrust)
            + self.up * (controls.vertical * VERTICAL_THRUST);
        self.velocity = self.velocity + acceleration * delta_time;
        self.velocity = self.velocity * (-DRAG * delta_time).exp();
        let limit = if controls.boost {
            MAX_BOOST_SPEED
        } else {
            MAX_SPEED
        } * self.speed_multiplier;
        let speed = self.velocity.length();
        if speed > limit {
            let target =
                speed + (limit - speed) * exponential_blend(SPEED_LIMIT_RESPONSE, delta_time);
            self.velocity = self.velocity * (target / speed);
        }
        self.position = self.position + self.velocity * delta_time;
    }

    fn update_evasive_loop(&mut self, delta_time: f32) {
        self.evasive_loop_remaining = (self.evasive_loop_remaining - delta_time).max(0.0);
        if self.evasive_loop_remaining < 1e-4 {
            self.evasive_loop_remaining = 0.0;
        }
        let progress = 1.0 - self.evasive_loop_remaining / EVASIVE_LOOP_DURATION;
        let right = self
            .evasive_loop_forward
            .cross(self.evasive_loop_up)
            .normalize();
        let angle = TAU * progress;
        self.forward = rotate_around(self.evasive_loop_forward, right, angle).normalize();
        self.up = rotate_around(self.evasive_loop_up, right, angle).normalize();
        self.velocity = self.forward * (EVASIVE_LOOP_SPEED * self.speed_multiplier);
        self.position = self.position + self.velocity * delta_time;
        if !self.is_evasive_looping() {
            self.set_orientation(self.evasive_loop_forward, self.evasive_loop_up);
            self.velocity = self.forward * (EVASIVE_LOOP_SPEED * self.speed_multiplier);
        }
    }

    pub fn enter_orbit(&mut self, body: usize, center: Vec3, body_radius: f32) {
        let offset = self.position - center;
        let altitude = Vec3::new(offset.x, 0.0, offset.z)
            .length()
            .max(body_radius * ORBIT_MIN_ALTITUDE_FACTOR);
        self.stop();
        self.mode = FlightMode::Orbit {
            body,
            altitude,
            height: offset.y,
            transition: 0.0,
        };
    }

    pub fn leave_orbit(&mut self) {
        if self.is_orbiting() && self.velocity.length() < ORBIT_RELEASE_SPEED {
            self.velocity = self.forward * ORBIT_RELEASE_SPEED;
        }
        self.mode = FlightMode::Free;
    }

    pub fn can_orbit(distance_to_center: f32, body_radius: f32) -> bool {
        distance_to_center <= body_radius * ORBIT_GRAB_RADIUS_FACTOR + ORBIT_GRAB_MARGIN
    }

    pub fn update_orbit(
        &mut self,
        controls: &ShipControls,
        center: Vec3,
        center_displacement: Vec3,
        body_radius: f32,
        orbit_capture_radius: f32,
        delta_time: f32,
    ) -> bool {
        let FlightMode::Orbit {
            body,
            mut altitude,
            height,
            mut transition,
        } = self.mode
        else {
            return false;
        };
        self.position = self.position + center_displacement;
        transition += (1.0 - transition) * exponential_blend(ORBIT_ENTRY_RESPONSE, delta_time);

        let minimum_altitude = body_radius * ORBIT_MIN_ALTITUDE_FACTOR;
        let exit_altitude = (orbit_capture_radius * ORBIT_EXIT_ALTITUDE_FACTOR + ORBIT_EXIT_MARGIN)
            .max(minimum_altitude + ORBIT_EXIT_MARGIN);
        altitude -= controls.thrust * ORBIT_ZOOM_RATE * transition * delta_time;
        if controls.thrust < 0.0 && altitude >= exit_altitude {
            let offset = self.position - center;
            let outward = Vec3::new(offset.x, 0.0, offset.z).normalize();
            self.velocity = outward * ORBIT_RELEASE_SPEED;
            self.mode = FlightMode::Free;
            return true;
        }
        altitude = altitude.clamp(minimum_altitude, exit_altitude);

        let offset = self.position - center;
        let horizontal = Vec3::new(offset.x, 0.0, offset.z);
        let radial = if horizontal.length() > f32::EPSILON {
            horizontal.normalize()
        } else {
            let forward = Vec3::new(self.forward.x, 0.0, self.forward.z);
            if forward.length() > f32::EPSILON {
                forward.normalize()
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            }
        };
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let angle = controls.yaw * ORBIT_ANGULAR_RATE * transition * delta_time;
        let new_radial = rotate_around(radial, world_up, angle).normalize();
        let previous = self.position;
        self.position = center + new_radial * altitude + world_up * height;

        let base_tangent = world_up.cross(new_radial).normalize();
        let projected_forward = Vec3::new(self.forward.x, 0.0, self.forward.z).normalize();
        let target_forward = if controls.yaw.abs() > f32::EPSILON {
            base_tangent * controls.yaw.signum()
        } else if projected_forward.length() > 0.5 {
            projected_forward
        } else {
            base_tangent
        };

        let blend = exponential_blend(ORBIT_ALIGN_RESPONSE, delta_time) * transition;
        let up = self.up + (world_up - self.up) * blend;
        let forward = self.forward + (target_forward - self.forward) * blend;
        self.set_orientation(forward, up);
        self.bank += (0.0 - self.bank) * exponential_blend(BANK_RESPONSE, delta_time);
        self.angular_velocity = Vec3::default();
        if delta_time > 0.0 {
            self.velocity = (self.position - previous) * (1.0 / delta_time);
        }
        self.mode = FlightMode::Orbit {
            body,
            altitude,
            height,
            transition,
        };
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evasive_loop_flies_a_vertical_circle_and_recovers_heading() {
        let mut ship = Ship::new(Vec3::default(), Vec3::new(0.0, 0.0, -1.0));
        ship.speed_multiplier = 1.5;
        let forward = ship.forward();

        assert!(ship.try_start_evasive_loop());
        assert!(ship.velocity.dot(forward) > 0.0);
        assert!(ship.is_evasive_loop_invulnerable());
        assert!(!ship.try_start_evasive_loop());

        let step = EVASIVE_LOOP_DURATION / 60.0;
        for _ in 0..15 {
            ship.update_free(&ShipControls::default(), step);
        }
        assert!(ship.forward().dot(ship.evasive_loop_up) > 0.9);
        assert!(ship.evasive_camera_forward().dot(forward) > 0.999);
        assert!(ship.evasive_camera_up().dot(ship.evasive_loop_up) > 0.999);
        assert!(ship.position.y > 0.0);

        for _ in 0..15 {
            ship.update_free(&ShipControls::default(), step);
        }
        assert!(ship.forward().dot(forward) < -0.9);
        assert!(!ship.is_evasive_loop_invulnerable());

        for _ in 0..30 {
            ship.update_free(&ShipControls::default(), step);
        }
        assert!(!ship.is_evasive_looping());
        assert!(ship.forward().dot(forward) > 0.999);
        assert!(!ship.try_start_evasive_loop());

        ship.update_free(&ShipControls::default(), EVASIVE_LOOP_COOLDOWN);
        assert!(ship.try_start_evasive_loop());
    }

    #[test]
    fn evasive_loop_cannot_start_while_orbiting() {
        let mut ship = Ship::new(Vec3::new(10.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        ship.enter_orbit(1, Vec3::default(), 2.0);

        assert!(!ship.try_start_evasive_loop());
    }
}
