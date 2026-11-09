use crate::engine::math::{Mat4, Vec3};
use crate::engine::transform::{basis_rotation, exponential_blend, rotate_around};

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
const ORBIT_MIN_SPEED: f32 = 1.5;
const ORBIT_MAX_SPEED: f32 = 12.0;
const ORBIT_DEFAULT_SPEED: f32 = 5.0;
const ORBIT_ACCELERATION: f32 = 6.0;
const ORBIT_CLIMB_RATE: f32 = 3.0;
const ORBIT_MIN_ALTITUDE_FACTOR: f32 = 1.35;
const ORBIT_MAX_ALTITUDE_FACTOR: f32 = 4.5;
const ORBIT_ALIGN_RESPONSE: f32 = 4.0;

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
        speed: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ship {
    pub position: Vec3,
    pub velocity: Vec3,
    forward: Vec3,
    up: Vec3,
    angular_velocity: Vec3,
    bank: f32,
    pub mode: FlightMode,
}

impl Ship {
    pub fn new(position: Vec3, forward: Vec3) -> Self {
        let rotation = basis_rotation(forward, Vec3::new(0.0, 1.0, 0.0));
        Self {
            position,
            velocity: Vec3::default(),
            forward: forward.normalize(),
            up: Vec3::new(
                rotation.rows[0][1],
                rotation.rows[1][1],
                rotation.rows[2][1],
            ),
            angular_velocity: Vec3::default(),
            bank: 0.0,
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

    pub fn is_orbiting(&self) -> bool {
        matches!(self.mode, FlightMode::Orbit { .. })
    }

    pub fn orbit_body(&self) -> Option<usize> {
        match self.mode {
            FlightMode::Orbit { body, .. } => Some(body),
            FlightMode::Free => None,
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
        self.steer(controls, delta_time);
        let thrust = if controls.boost { BOOST_THRUST } else { THRUST };
        let acceleration = self.forward * (controls.thrust * thrust)
            + self.up * (controls.vertical * VERTICAL_THRUST);
        self.velocity = self.velocity + acceleration * delta_time;
        self.velocity = self.velocity * (-DRAG * delta_time).exp();
        let limit = if controls.boost {
            MAX_BOOST_SPEED
        } else {
            MAX_SPEED
        };
        let speed = self.velocity.length();
        if speed > limit {
            let target =
                speed + (limit - speed) * exponential_blend(SPEED_LIMIT_RESPONSE, delta_time);
            self.velocity = self.velocity * (target / speed);
        }
        self.position = self.position + self.velocity * delta_time;
    }

    pub fn enter_orbit(&mut self, body: usize, center: Vec3, body_radius: f32) {
        let altitude = (self.position - center).length().clamp(
            body_radius * ORBIT_MIN_ALTITUDE_FACTOR,
            body_radius * ORBIT_MAX_ALTITUDE_FACTOR,
        );
        let speed = self
            .velocity
            .length()
            .clamp(ORBIT_DEFAULT_SPEED, ORBIT_MAX_SPEED);
        self.mode = FlightMode::Orbit {
            body,
            altitude,
            speed,
        };
    }

    pub fn leave_orbit(&mut self) {
        if let FlightMode::Orbit { speed, .. } = self.mode {
            self.velocity = self.forward * speed;
        }
        self.mode = FlightMode::Free;
    }

    pub fn can_orbit(distance_to_center: f32, body_radius: f32) -> bool {
        distance_to_center <= body_radius * ORBIT_MAX_ALTITUDE_FACTOR + 6.0
    }

    pub fn update_orbit(
        &mut self,
        controls: &ShipControls,
        center: Vec3,
        center_displacement: Vec3,
        body_radius: f32,
        delta_time: f32,
    ) {
        let FlightMode::Orbit {
            body,
            mut altitude,
            mut speed,
        } = self.mode
        else {
            return;
        };
        self.position = self.position + center_displacement;
        speed = (speed + controls.thrust * ORBIT_ACCELERATION * delta_time)
            .clamp(ORBIT_MIN_SPEED, ORBIT_MAX_SPEED);
        altitude = (altitude + controls.vertical * ORBIT_CLIMB_RATE * delta_time).clamp(
            body_radius * ORBIT_MIN_ALTITUDE_FACTOR,
            body_radius * ORBIT_MAX_ALTITUDE_FACTOR,
        );

        let radial = (self.position - center).normalize();
        let heading = rotate_around(self.forward, radial, controls.yaw * YAW_RATE * delta_time);
        let tangent = (heading - radial * heading.dot(radial)).normalize();
        let target_altitude = center + radial * altitude;
        let previous = self.position;
        let moved = target_altitude + tangent * (speed * delta_time);
        self.position = center + (moved - center).normalize() * altitude;
        let new_radial = (self.position - center).normalize();
        let new_tangent = (tangent - new_radial * tangent.dot(new_radial)).normalize();

        let blend = exponential_blend(ORBIT_ALIGN_RESPONSE, delta_time);
        let up = self.up + (new_radial - self.up) * blend;
        self.set_orientation(new_tangent, up);
        let target_bank = controls.yaw * MAX_BANK;
        self.bank += (target_bank - self.bank) * exponential_blend(BANK_RESPONSE, delta_time);
        self.angular_velocity = Vec3::default();
        if delta_time > 0.0 {
            self.velocity = (self.position - previous) * (1.0 / delta_time);
        }
        self.mode = FlightMode::Orbit {
            body,
            altitude,
            speed,
        };
    }
}
