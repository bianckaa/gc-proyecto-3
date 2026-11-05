use super::player::Ship;
use crate::engine::math::Vec3;
use crate::engine::transform::{lerp, smoothstep, WORLD_UP};
use crate::renderer::warp_effect::WarpEffect;
use std::f32::consts::PI;

pub const WARP_TRAVEL_TIME: f32 = 2.2;
pub const WARP_COOLDOWN: f32 = 1.5;
const ARRIVAL_DISTANCE_FACTOR: f32 = 2.6;
const ARRIVAL_ELEVATION: f32 = 0.35;
const ARRIVAL_MARGIN: f32 = 4.0;
const ARC_FACTOR: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Destination {
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WarpState {
    Ready,
    Traveling {
        target: usize,
        start: Vec3,
        start_forward: Vec3,
        elapsed: f32,
    },
    Cooldown {
        remaining: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpEvent {
    None,
    Arrived(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Warp {
    state: WarpState,
}

impl Default for Warp {
    fn default() -> Self {
        Self::new()
    }
}

pub fn safe_distance(radius: f32) -> f32 {
    radius * ARRIVAL_DISTANCE_FACTOR + ARRIVAL_MARGIN
}

pub fn arrival_point(start: Vec3, destination: Destination) -> Vec3 {
    let offset = start - destination.center;
    let horizontal = if offset.length() > f32::EPSILON {
        offset.normalize()
    } else {
        Vec3::new(0.0, 0.0, 1.0)
    };
    let direction = (horizontal + WORLD_UP * ARRIVAL_ELEVATION).normalize();
    destination.center + direction * safe_distance(destination.radius)
}

impl Warp {
    pub fn new() -> Self {
        Self {
            state: WarpState::Ready,
        }
    }

    pub fn state(&self) -> WarpState {
        self.state
    }

    pub fn is_ready(&self) -> bool {
        self.state == WarpState::Ready
    }

    pub fn is_traveling(&self) -> bool {
        matches!(self.state, WarpState::Traveling { .. })
    }

    pub fn target(&self) -> Option<usize> {
        match self.state {
            WarpState::Traveling { target, .. } => Some(target),
            _ => None,
        }
    }

    pub fn progress(&self) -> f32 {
        match self.state {
            WarpState::Traveling { elapsed, .. } => (elapsed / WARP_TRAVEL_TIME).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }

    pub fn engage(&mut self, target: usize, ship: &mut Ship, effect: &mut WarpEffect) -> bool {
        if !self.is_ready() {
            return false;
        }
        if ship.is_orbiting() {
            ship.leave_orbit();
        }
        effect.start_warp();
        self.state = WarpState::Traveling {
            target,
            start: ship.position,
            start_forward: ship.forward(),
            elapsed: 0.0,
        };
        true
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        ship: &mut Ship,
        destination: Destination,
    ) -> WarpEvent {
        match self.state {
            WarpState::Ready => WarpEvent::None,
            WarpState::Cooldown { remaining } => {
                let remaining = remaining - delta_time;
                self.state = if remaining <= 0.0 {
                    WarpState::Ready
                } else {
                    WarpState::Cooldown { remaining }
                };
                WarpEvent::None
            }
            WarpState::Traveling {
                target,
                start,
                start_forward,
                elapsed,
            } => {
                let elapsed = elapsed + delta_time;
                let t = (elapsed / WARP_TRAVEL_TIME).clamp(0.0, 1.0);
                let eased = smoothstep(t);
                let arrival = arrival_point(start, destination);
                let lift = WORLD_UP * ((PI * t).sin() * (arrival - start).length() * ARC_FACTOR);
                ship.position = lerp(start, arrival, eased) + lift;
                let facing = (destination.center - ship.position).normalize();
                let forward = lerp(start_forward, facing, smoothstep(t * 1.4));
                ship.set_orientation(forward, ship.up() + WORLD_UP * 0.5);
                ship.stop();
                if t >= 1.0 {
                    ship.position = arrival;
                    ship.set_orientation(facing, ship.up() + WORLD_UP * 0.5);
                    self.state = WarpState::Cooldown {
                        remaining: WARP_COOLDOWN,
                    };
                    WarpEvent::Arrived(target)
                } else {
                    self.state = WarpState::Traveling {
                        target,
                        start,
                        start_forward,
                        elapsed,
                    };
                    WarpEvent::None
                }
            }
        }
    }
}
