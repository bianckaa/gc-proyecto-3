use gc_proyecto_3::engine::camera::Camera;
use gc_proyecto_3::engine::math::{Mat4, Vec2, Vec3};
use gc_proyecto_3::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gc_proyecto_3::renderer::lighting::PointLight;
use gc_proyecto_3::renderer::obj::load_normalized;
use gc_proyecto_3::renderer::shaders::ShaderKind;
use gc_proyecto_3::renderer::{
    Deformation, LodGroup, Material, Mesh, RenderObject, Renderer, Vertex,
};
use raylib::prelude::*;
use std::f32::consts::{PI, TAU};

const WINDOW_SCALE: i32 = 2;
const LOD_SUBDIVISIONS: [u32; 3] = [4, 3, 1];
const AMBIENT_LIGHT: f32 = 0.3;
const ORBIT_SEGMENTS: usize = 96;
const ORBIT_RADIUS: f32 = 13.0;
const ASTEROID_COUNT: usize = 70;
const CRYSTAL_COUNT: usize = 8;
const FIELD_SEED: u32 = 4242;
const FIELD_INNER_RADIUS: f32 = 14.0;
const FIELD_WIDTH: f32 = 28.0;
const FIELD_HEIGHT: f32 = 12.0;
const ETA_OBJ: &str = include_str!("../assets/models/ETA_2.obj");
const ENEMY_OBJ: &str = include_str!("../assets/models/ENEMY1.obj");
const PLAYER_MODEL_SCALE: f32 = 1.45;
const MODEL_FORWARD_CORRECTION: f32 = PI;

const CRUISE_SPEED: f32 = 5.0;
const BOOST_SPEED: f32 = 10.0;
const YAW_SPEED: f32 = 1.3;
const PITCH_SPEED: f32 = 0.9;
const MAX_PITCH: f32 = 1.2;
const MAX_BANK: f32 = 0.6;
const BANK_RESPONSE: f32 = 5.0;

const FOLLOW_DISTANCE: f32 = 5.5;
const FOLLOW_HEIGHT: f32 = 2.2;
const FOLLOW_LOOK_AHEAD: f32 = 8.0;
const FOLLOW_SMOOTHING: f32 = 6.0;

const LASER_SPEED: f32 = 45.0;
const LASER_LENGTH: f32 = 3.0;
const LASER_LIFETIME: f32 = 1.2;
const LASER_INTERVAL: f32 = 1.5;

const PLAYER_COLOR: Vec3 = Vec3::new(0.96, 0.66, 0.08);
const ENEMY_COLOR: Vec3 = Vec3::new(0.46, 0.11, 0.68);
const SAND_COLOR: Vec3 = Vec3::new(0.85, 0.76, 0.6);
const CRYSTAL_COLOR: Vec3 = Vec3::new(0.75, 0.31, 1.0);
const ORBIT_COLOR: Color = Color::new(192, 80, 255);
const LASER_COLOR: Color = Color::new(255, 48, 64);
const WHITE: Color = Color::new(255, 255, 255);
const METRICS_COLOR: Color = Color::new(92, 255, 92);

const ROCK_DEFORMATION: Deformation = Deformation {
    seed: 11,
    amplitude: 0.35,
    frequency: 2.5,
    crater_count: 6,
    crater_radius: 0.45,
    crater_depth: 0.12,
};

struct Planet {
    orbit_radius: f32,
    orbit_speed: f32,
    start_angle: f32,
    radius: f32,
    material: Material,
}

struct FieldObject {
    position: Vec3,
    scale: Vec3,
    spin_speed: f32,
}

struct Enemy {
    orbit_radius: f32,
    orbit_speed: f32,
    start_angle: f32,
    height: f32,
    scale: f32,
}

struct Laser {
    position: Vec3,
    direction: Vec3,
    age: f32,
}

struct Ship {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    bank: f32,
}

impl Ship {
    fn orientation(&self) -> Mat4 {
        Mat4::rotation_y(self.yaw) * Mat4::rotation_x(self.pitch)
    }

    fn forward(&self) -> Vec3 {
        (self.orientation() * Vec3::new(0.0, 0.0, -1.0).extend(0.0)).truncate()
    }

    fn up(&self) -> Vec3 {
        (self.orientation() * Vec3::new(0.0, 1.0, 0.0).extend(0.0)).truncate()
    }

    fn model_matrix(&self) -> Mat4 {
        Mat4::translation(self.position) * self.orientation() * Mat4::rotation_z(self.bank)
    }
}

struct Random {
    state: u32,
}

impl Random {
    fn next(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.state >> 8) as f32 / (1u32 << 24) as f32
    }
}

fn push_triangle(mesh: &mut Mesh, corners: [Vec3; 3]) {
    let [a, b, c] = corners;
    let normal = (b - a).cross(c - a).normalize();
    let center = (a + b + c) * (1.0 / 3.0);
    let (ordered, normal) = if normal.dot(center) >= 0.0 {
        ([a, b, c], normal)
    } else {
        ([a, c, b], -normal)
    };
    for position in ordered {
        mesh.indices.push(mesh.vertices.len() as u32);
        mesh.vertices.push(Vertex {
            position,
            normal,
            uv: Vec2::default(),
        });
    }
}

fn bipyramid_mesh(front: Vec3, back: Vec3, ring: [Vec3; 4]) -> Mesh {
    let mut mesh = Mesh::default();
    for index in 0..ring.len() {
        let next = ring[(index + 1) % ring.len()];
        push_triangle(&mut mesh, [front, ring[index], next]);
        push_triangle(&mut mesh, [back, ring[index], next]);
    }
    mesh
}

fn crystal_mesh() -> Mesh {
    bipyramid_mesh(
        Vec3::new(0.0, 0.0, -1.6),
        Vec3::new(0.0, 0.0, 1.6),
        [
            Vec3::new(0.35, 0.0, 0.0),
            Vec3::new(0.0, 0.2, 0.0),
            Vec3::new(-0.35, 0.0, 0.0),
            Vec3::new(0.0, -0.2, 0.0),
        ],
    )
}

fn enemies() -> [Enemy; 5] {
    [
        Enemy {
            orbit_radius: 17.0,
            orbit_speed: 0.16,
            start_angle: PI * 0.5,
            height: 1.4,
            scale: 1.15,
        },
        Enemy {
            orbit_radius: 20.0,
            orbit_speed: 0.11,
            start_angle: 0.25,
            height: -1.8,
            scale: 1.05,
        },
        Enemy {
            orbit_radius: 22.5,
            orbit_speed: 0.13,
            start_angle: 2.55,
            height: 3.0,
            scale: 1.25,
        },
        Enemy {
            orbit_radius: 25.0,
            orbit_speed: 0.09,
            start_angle: 4.10,
            height: -3.2,
            scale: 1.10,
        },
        Enemy {
            orbit_radius: 28.0,
            orbit_speed: 0.07,
            start_angle: 5.35,
            height: 0.6,
            scale: 1.30,
        },
    ]
}

fn material(base_color: Vec3, shader: ShaderKind) -> Material {
    Material {
        base_color,
        emissive: Vec3::default(),
        shader,
        outline: true,
    }
}

fn planets() -> [Planet; 3] {
    [
        Planet {
            orbit_radius: 5.5,
            orbit_speed: 0.5,
            start_angle: 1.2,
            radius: 0.9,
            material: material(Vec3::new(1.0, 0.25, 0.1), ShaderKind::Lava),
        },
        Planet {
            orbit_radius: 9.0,
            orbit_speed: 0.3,
            start_angle: 2.4,
            radius: 1.1,
            material: material(Vec3::new(0.25, 0.75, 1.0), ShaderKind::Toon),
        },
        Planet {
            orbit_radius: ORBIT_RADIUS,
            orbit_speed: 0.2,
            start_angle: 0.9,
            radius: 1.7,
            material: material(Vec3::new(1.0, 0.7, 0.4), ShaderKind::GasGiant),
        },
    ]
}

fn scatter(random: &mut Random, count: usize, min_size: f32, size_range: f32) -> Vec<FieldObject> {
    (0..count)
        .map(|_| {
            let angle = random.next() * TAU;
            let radius = FIELD_INNER_RADIUS + random.next() * FIELD_WIDTH;
            let height = (random.next() - 0.5) * FIELD_HEIGHT;
            let size = min_size + random.next() * random.next() * size_range;
            FieldObject {
                position: Vec3::new(radius * angle.cos(), height, radius * angle.sin()),
                scale: Vec3::new(size, size * (0.6 + random.next() * 0.5), size),
                spin_speed: 0.2 + random.next() * 0.6,
            }
        })
        .collect()
}

fn orbit_point(radius: f32, angle: f32) -> Vec3 {
    Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin())
}

fn axis(rl: &RaylibHandle, positive: KeyboardKey, negative: KeyboardKey) -> f32 {
    rl.is_key_down(positive) as i32 as f32 - rl.is_key_down(negative) as i32 as f32
}

fn fire_lasers(lasers: &mut Vec<Laser>, ship: &Ship) {
    let side = ship.forward().cross(ship.up()).normalize();
    for offset in [-0.9, 0.9] {
        lasers.push(Laser {
            position: ship.position + side * offset,
            direction: ship.forward(),
            age: 0.0,
        });
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(
            FRAMEBUFFER_WIDTH as i32 * WINDOW_SCALE,
            FRAMEBUFFER_HEIGHT as i32 * WINDOW_SCALE,
        )
        .title("Renderer demo")
        .build();

    let image = Image::gen_image_color(
        FRAMEBUFFER_WIDTH as i32,
        FRAMEBUFFER_HEIGHT as i32,
        raylib::color::Color::BLACK,
    );
    let mut texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("no se pudo crear la textura");
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

    let mut renderer = Renderer::new();
    renderer.light = PointLight::new(Vec3::default(), AMBIENT_LIGHT);

    let sphere = LodGroup::new(|level| Mesh::icosphere(LOD_SUBDIVISIONS[level]));
    let rock = LodGroup::new(|level| {
        let mut mesh = Mesh::icosphere(LOD_SUBDIVISIONS[level]);
        mesh.deform(&ROCK_DEFORMATION);
        mesh
    });
    let ship_model = load_normalized(ETA_OBJ).expect("no se pudo cargar assets/models/ETA_2.obj");
    let enemy_model =
        load_normalized(ENEMY_OBJ).expect("no se pudo cargar assets/models/ENEMY1.obj");
    let crystal = crystal_mesh();

    let planets = planets();
    let sun_material = Material {
        emissive: Vec3::new(0.1, 0.05, 0.0),
        ..material(Vec3::new(1.0, 0.65, 0.15), ShaderKind::Sun)
    };
    let asteroid_material = material(SAND_COLOR, ShaderKind::Rocky);
    let crystal_material = material(CRYSTAL_COLOR, ShaderKind::Lambert);
    let ship_material = material(PLAYER_COLOR, ShaderKind::EtaPlayer);
    let enemy_material = material(ENEMY_COLOR, ShaderKind::EnemyShip);

    let mut random = Random { state: FIELD_SEED };
    let asteroids = scatter(&mut random, ASTEROID_COUNT, 0.3, 2.2);
    let crystals = scatter(&mut random, CRYSTAL_COUNT, 0.7, 0.6);
    let enemies = enemies();

    let mut ship = Ship {
        position: Vec3::new(0.0, 2.5, 30.0),
        yaw: 0.0,
        pitch: -0.05,
        bank: 0.0,
    };
    let aspect = FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32;
    let mut camera = Camera::new(
        ship.position - ship.forward() * FOLLOW_DISTANCE + ship.up() * FOLLOW_HEIGHT,
        aspect,
    );
    let mut lasers: Vec<Laser> = Vec::new();
    let mut next_laser = 0.0_f32;
    let mut show_metrics = true;
    let mut time = 0.0_f32;

    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        time += delta_time;

        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            renderer.dithering_enabled = !renderer.dithering_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_O) {
            renderer.outline_enabled = !renderer.outline_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            renderer.culling_enabled = !renderer.culling_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_L) {
            renderer.lod_enabled = !renderer.lod_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            show_metrics = !show_metrics;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            renderer.warp.start_warp();
        }
        renderer.warp.update(delta_time);
        renderer.metrics.update(delta_time);

        let turn = axis(&rl, KeyboardKey::KEY_A, KeyboardKey::KEY_D);
        let climb = axis(&rl, KeyboardKey::KEY_UP, KeyboardKey::KEY_DOWN);
        ship.yaw += turn * YAW_SPEED * delta_time;
        ship.pitch = (ship.pitch + climb * PITCH_SPEED * delta_time).clamp(-MAX_PITCH, MAX_PITCH);
        let target_bank = turn * MAX_BANK;
        ship.bank += (target_bank - ship.bank) * (1.0 - (-BANK_RESPONSE * delta_time).exp());
        let speed = if rl.is_key_down(KeyboardKey::KEY_W) {
            BOOST_SPEED
        } else if rl.is_key_down(KeyboardKey::KEY_S) {
            0.0
        } else {
            CRUISE_SPEED
        };
        ship.position = ship.position + ship.forward() * (speed * delta_time);

        camera.follow(
            ship.position,
            ship.forward(),
            ship.up(),
            FOLLOW_DISTANCE,
            FOLLOW_HEIGHT,
            FOLLOW_LOOK_AHEAD,
            FOLLOW_SMOOTHING,
            delta_time,
        );

        if rl.is_key_pressed(KeyboardKey::KEY_F) || time >= next_laser {
            fire_lasers(&mut lasers, &ship);
            next_laser = time + LASER_INTERVAL;
        }
        for laser in &mut lasers {
            laser.position = laser.position + laser.direction * (LASER_SPEED * delta_time);
            laser.age += delta_time;
        }
        lasers.retain(|laser| laser.age < LASER_LIFETIME);
        for laser in &lasers {
            let tail = laser.position - laser.direction * LASER_LENGTH;
            renderer.draw_line_3d(tail, laser.position, LASER_COLOR, false);
        }

        for segment in (0..ORBIT_SEGMENTS).step_by(2) {
            let step = TAU / ORBIT_SEGMENTS as f32;
            renderer.draw_line_3d(
                orbit_point(ORBIT_RADIUS, segment as f32 * step),
                orbit_point(ORBIT_RADIUS, (segment + 1) as f32 * step),
                ORBIT_COLOR,
                false,
            );
        }

        let spin = Mat4::rotation_y(time * 0.4);
        let mut objects = vec![
            RenderObject {
                mesh: &sphere.levels[0],
                lod: Some(&sphere),
                model_matrix: spin * Mat4::scale(Vec3::new(3.0, 3.0, 3.0)),
                material: sun_material,
                bounding_radius: 3.0,
            },
            RenderObject {
                mesh: &ship_model,
                lod: None,
                model_matrix: ship.model_matrix()
                    * Mat4::rotation_y(MODEL_FORWARD_CORRECTION)
                    * Mat4::scale(Vec3::new(
                        PLAYER_MODEL_SCALE,
                        PLAYER_MODEL_SCALE,
                        PLAYER_MODEL_SCALE,
                    )),
                material: ship_material,
                bounding_radius: PLAYER_MODEL_SCALE,
            },
        ];
        for planet in &planets {
            let center = orbit_point(
                planet.orbit_radius,
                planet.start_angle + time * planet.orbit_speed,
            );
            let size = Vec3::new(planet.radius, planet.radius, planet.radius);
            objects.push(RenderObject {
                mesh: &sphere.levels[0],
                lod: Some(&sphere),
                model_matrix: Mat4::translation(center) * spin * Mat4::scale(size),
                material: planet.material,
                bounding_radius: planet.radius,
            });
        }
        for asteroid in &asteroids {
            objects.push(RenderObject {
                mesh: &rock.levels[0],
                lod: Some(&rock),
                model_matrix: Mat4::translation(asteroid.position)
                    * Mat4::rotation_y(time * asteroid.spin_speed)
                    * Mat4::rotation_x(time * asteroid.spin_speed * 0.7)
                    * Mat4::scale(asteroid.scale),
                material: asteroid_material,
                bounding_radius: asteroid.scale.x * 1.5,
            });
        }
        for piece in &crystals {
            objects.push(RenderObject {
                mesh: &crystal,
                lod: None,
                model_matrix: Mat4::translation(piece.position)
                    * Mat4::rotation_y(time * piece.spin_speed + PI * piece.spin_speed)
                    * Mat4::scale(piece.scale),
                material: crystal_material,
                bounding_radius: piece.scale.x * 1.7,
            });
        }
        for enemy in &enemies {
            let angle = enemy.start_angle - time * enemy.orbit_speed;
            let position =
                orbit_point(enemy.orbit_radius, angle) + Vec3::new(0.0, enemy.height, 0.0);
            let scale = Vec3::new(enemy.scale, enemy.scale, enemy.scale);
            objects.push(RenderObject {
                mesh: &enemy_model,
                lod: None,
                model_matrix: Mat4::translation(position)
                    * Mat4::rotation_y(-angle)
                    * Mat4::rotation_y(MODEL_FORWARD_CORRECTION)
                    * Mat4::scale(scale),
                material: enemy_material,
                bounding_radius: enemy.scale,
            });
        }

        renderer.render(&objects, &camera, time);
        renderer.draw_crosshair(WHITE);
        if show_metrics {
            renderer.draw_metrics(METRICS_COLOR);
        }

        texture
            .update_texture(renderer.pixels())
            .expect("no se pudo actualizar la textura");

        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(raylib::color::Color::BLACK);
        drawing.draw_texture_ex(
            &texture,
            Vector2::new(0.0, 0.0),
            0.0,
            WINDOW_SCALE as f32,
            raylib::color::Color::WHITE,
        );
    }
}
