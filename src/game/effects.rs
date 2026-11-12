use crate::engine::math::Vec3;
use crate::game::run_world::PlanetKind;
use crate::renderer::framebuffer::Color;
use crate::renderer::Renderer;
use crate::world::generation::SeededRandom;
use std::f32::consts::TAU;

const SPARK_TRAIL: f32 = 0.06;
const SPARK_DRAG: f32 = 1.6;
const RING_SEGMENTS: usize = 32;
const MAX_SPARKS: usize = 1500;
const KNOCKBACK_STREAKS: usize = 6;
const KNOCKBACK_SPEED: f32 = 22.0;
const KNOCKBACK_REFERENCE: f32 = 12.0;
const KNOCKBACK_COLOR: Color = Color::new(240, 240, 255);

const IMPACT_COLOR: Color = Color::new(255, 220, 160);
const ENEMY_COLORS: [Color; 2] = [Color::new(192, 80, 255), Color::new(255, 150, 60)];
const ELITE_COLORS: [Color; 2] = [Color::new(255, 50, 40), Color::new(255, 200, 120)];
const PLAYER_COLORS: [Color; 2] = [Color::new(92, 255, 92), Color::new(255, 255, 255)];

struct Spark {
    position: Vec3,
    velocity: Vec3,
    life: f32,
    max_life: f32,
    color: Color,
}

struct Ring {
    center: Vec3,
    u_axis: Vec3,
    v_axis: Vec3,
    max_radius: f32,
    life: f32,
    max_life: f32,
    color: Color,
}

pub struct Effects {
    sparks: Vec<Spark>,
    rings: Vec<Ring>,
    random: SeededRandom,
}

fn scaled(color: Color, amount: f32) -> Color {
    let channel = |value: u8| (value as f32 * amount.clamp(0.0, 1.0)) as u8;
    Color::new(channel(color.r), channel(color.g), channel(color.b))
}

pub fn planet_colors(kind: PlanetKind) -> [Color; 2] {
    match kind {
        PlanetKind::Rocky => [Color::new(217, 194, 154), Color::new(255, 140, 60)],
        PlanetKind::Gas => [Color::new(120, 190, 255), Color::new(230, 240, 255)],
        PlanetKind::Core => [Color::new(255, 110, 40), Color::new(255, 230, 120)],
        PlanetKind::Droid => [Color::new(200, 205, 215), Color::new(80, 230, 255)],
    }
}

impl Effects {
    pub fn new() -> Self {
        Self {
            sparks: Vec::new(),
            rings: Vec::new(),
            random: SeededRandom::new(0x5EED_F1A5),
        }
    }

    pub fn clear(&mut self) {
        self.sparks.clear();
        self.rings.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.sparks.is_empty() && self.rings.is_empty()
    }

    fn random_direction(&mut self) -> Vec3 {
        let height = self.random.range(-1.0, 1.0);
        let angle = self.random.range(0.0, TAU);
        let ring = (1.0 - height * height).sqrt();
        Vec3::new(ring * angle.cos(), height, ring * angle.sin())
    }

    pub fn burst(&mut self, center: Vec3, colors: [Color; 2], count: usize, speed: f32, life: f32) {
        for index in 0..count {
            if self.sparks.len() >= MAX_SPARKS {
                break;
            }
            let direction = self.random_direction();
            let velocity = direction * (speed * self.random.range(0.35, 1.0));
            let spark_life = life * self.random.range(0.5, 1.0);
            self.sparks.push(Spark {
                position: center,
                velocity,
                life: spark_life,
                max_life: spark_life,
                color: colors[index % 2],
            });
        }
    }

    pub fn ring(&mut self, center: Vec3, max_radius: f32, color: Color, life: f32) {
        let normal = self.random_direction();
        let helper = if normal.y.abs() < 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let u_axis = normal.cross(helper).normalize();
        let v_axis = normal.cross(u_axis);
        self.rings.push(Ring {
            center,
            u_axis,
            v_axis,
            max_radius,
            life,
            max_life: life,
            color,
        });
    }

    pub fn impact(&mut self, position: Vec3) {
        self.burst(position, [IMPACT_COLOR, IMPACT_COLOR], 5, 9.0, 0.25);
    }

    pub fn knockback(&mut self, position: Vec3, impulse: Vec3) {
        let length = impulse.length();
        if length <= f32::EPSILON {
            return;
        }
        let direction = impulse * (1.0 / length);
        let strength = (length / KNOCKBACK_REFERENCE).clamp(0.4, 2.0);
        for _ in 0..KNOCKBACK_STREAKS {
            if self.sparks.len() >= MAX_SPARKS {
                break;
            }
            let jitter = self.random_direction() * 0.35;
            let velocity = (direction + jitter).normalize() * (-KNOCKBACK_SPEED * strength);
            let life = self.random.range(0.15, 0.3);
            self.sparks.push(Spark {
                position,
                velocity,
                life,
                max_life: life,
                color: KNOCKBACK_COLOR,
            });
        }
        self.ring(position, 1.2 * strength, KNOCKBACK_COLOR, 0.2);
    }

    pub fn tough_enemy_explosion(&mut self, position: Vec3, elite: bool) {
        let (colors, count, speed, ring) = if elite {
            (ELITE_COLORS, 70, 20.0, 7.0)
        } else {
            (ENEMY_COLORS, 40, 16.0, 4.5)
        };
        self.burst(position, colors, count, speed, 0.9);
        self.ring(position, ring, colors[0], 0.6);
    }

    pub fn enemy_explosion(&mut self, position: Vec3) {
        self.burst(position, ENEMY_COLORS, 26, 14.0, 0.7);
        self.ring(position, 3.5, ENEMY_COLORS[0], 0.45);
    }

    pub fn enemy_arrival(&mut self, position: Vec3) {
        self.ring(position, 3.0, ENEMY_COLORS[0], 0.6);
        self.burst(
            position,
            [ENEMY_COLORS[0], Color::new(255, 255, 255)],
            10,
            5.0,
            0.45,
        );
    }

    pub fn bomb_explosion(&mut self, position: Vec3, radius: f32) {
        self.burst(position, [Color::new(255, 140, 40), Color::new(255, 240, 200)], 40, radius * 2.5, 0.6);
        self.ring(position, radius, Color::new(255, 170, 60), 0.4);
    }

    pub fn planet_explosion(&mut self, position: Vec3, radius: f32, kind: PlanetKind) {
        let colors = planet_colors(kind);
        self.burst(position, colors, 140, radius * 6.0, 1.8);
        self.ring(position, radius * 3.5, colors[0], 1.2);
        self.ring(position, radius * 2.2, colors[1], 0.9);
    }

    pub fn player_explosion(&mut self, position: Vec3) {
        self.burst(position, PLAYER_COLORS, 90, 18.0, 1.6);
        self.ring(position, 6.0, PLAYER_COLORS[0], 1.0);
    }

    pub fn update(&mut self, delta_time: f32) {
        let drag = (-SPARK_DRAG * delta_time).exp();
        self.sparks.retain_mut(|spark| {
            spark.life -= delta_time;
            spark.position = spark.position + spark.velocity * delta_time;
            spark.velocity = spark.velocity * drag;
            spark.life > 0.0
        });
        self.rings.retain_mut(|ring| {
            ring.life -= delta_time;
            ring.life > 0.0
        });
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        for spark in &self.sparks {
            let fade = spark.life / spark.max_life;
            let tail = spark.position - spark.velocity * SPARK_TRAIL;
            renderer.draw_line_3d(tail, spark.position, scaled(spark.color, 0.4 + 0.6 * fade), false);
        }
        for ring in &self.rings {
            let progress = 1.0 - ring.life / ring.max_life;
            let radius = ring.max_radius * (1.0 - (1.0 - progress).powi(3));
            let color = scaled(ring.color, 1.0 - progress * 0.7);
            let points: Vec<Vec3> = (0..=RING_SEGMENTS)
                .map(|index| {
                    let angle = index as f32 / RING_SEGMENTS as f32 * TAU;
                    ring.center + (ring.u_axis * angle.cos() + ring.v_axis * angle.sin()) * radius
                })
                .collect();
            for segment in points.windows(2).step_by(2) {
                renderer.draw_line_3d(segment[0], segment[1], color, false);
            }
        }
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knockback_leaves_streaks_behind_the_target() {
        let mut effects = Effects::new();
        effects.knockback(Vec3::default(), Vec3::new(0.0, 0.0, -12.0));
        assert!(effects.sparks.iter().all(|spark| spark.velocity.z > 0.0));
        effects.knockback(Vec3::default(), Vec3::default());
        assert_eq!(effects.rings.len(), 1);
    }

    #[test]
    fn explosions_fade_out_over_time() {
        let mut effects = Effects::new();
        effects.enemy_explosion(Vec3::default());
        effects.planet_explosion(Vec3::default(), 3.0, PlanetKind::Gas);
        assert!(!effects.is_empty());
        for _ in 0..240 {
            effects.update(1.0 / 60.0);
        }
        assert!(effects.is_empty());
    }
}
