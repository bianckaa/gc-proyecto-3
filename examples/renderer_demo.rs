use gc_proyecto_3::engine::camera::Camera;
use gc_proyecto_3::engine::math::{Mat4, Vec3};
use gc_proyecto_3::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gc_proyecto_3::renderer::shaders::ShaderKind;
use gc_proyecto_3::renderer::{Deformation, LodGroup, Material, Mesh, RenderObject, Renderer};
use raylib::prelude::*;
use std::f32::consts::TAU;

const WINDOW_SCALE: i32 = 2;
const LOD_SUBDIVISIONS: [u32; 3] = [4, 3, 1];
const ASTEROID_COUNT: usize = 24;
const ASTEROID_BELT_RADIUS: f32 = 15.0;
const ORBIT_SEGMENTS: usize = 96;
const MOVE_SPEED: f32 = 8.0;
const TURN_SPEED: f32 = 1.5;
const SPIN_SPEED: f32 = 0.4;
const INITIAL_PITCH: f32 = -0.42;

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


fn material(base_color: Vec3, shader: ShaderKind) -> Material {
    Material {
        base_color,
        emissive: Vec3::default(),
        shader,
        outline: true,
    }
}

fn planets() -> [Planet; 4] {
    [
        Planet {
            orbit_radius: 4.5,
            orbit_speed: 0.5,
            start_angle: 0.6,
            radius: 0.9,
            material: material(Vec3::new(1.0, 0.25, 0.1), ShaderKind::Lava),
        },
        Planet {
            orbit_radius: 7.0,
            orbit_speed: 0.3,
            start_angle: 2.4,
            radius: 1.0,
            material: material(Vec3::new(0.25, 0.75, 1.0), ShaderKind::Toon),
        },
        Planet {
            orbit_radius: 9.5,
            orbit_speed: 0.2,
            start_angle: 4.0,
            radius: 1.6,
            material: material(Vec3::new(1.0, 0.7, 0.4), ShaderKind::GasGiant),
        },
        Planet {
            orbit_radius: 12.0,
            orbit_speed: 0.15,
            start_angle: 5.4,
            radius: 0.8,
            material: material(Vec3::new(0.85, 0.76, 0.6), ShaderKind::Rocky),
        },
    ]
}

fn orbit_point(radius: f32, angle: f32) -> Vec3 {
    Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin())
}

fn axis(rl: &RaylibHandle, positive: KeyboardKey, negative: KeyboardKey) -> f32 {
    rl.is_key_down(positive) as i32 as f32 - rl.is_key_down(negative) as i32 as f32
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
    let sphere = LodGroup::new(|level| Mesh::icosphere(LOD_SUBDIVISIONS[level]));
    let rock = LodGroup::new(|level| {
        let mut mesh = Mesh::icosphere(LOD_SUBDIVISIONS[level]);
        mesh.deform(&ROCK_DEFORMATION);
        mesh
    });
    let planets = planets();
    let sun_material = Material {
        emissive: Vec3::new(0.1, 0.05, 0.0),
        ..material(Vec3::new(1.0, 0.65, 0.15), ShaderKind::Sun)
    };

    let asteroid_material = material(Vec3::new(0.85, 0.76, 0.6), ShaderKind::Rocky);

    let aspect = FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32;
    let mut camera = Camera::new(Vec3::new(0.0, 11.0, 20.0), aspect);
    camera.rotate(0.0, INITIAL_PITCH, 0.0);
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

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            renderer.warp.start_warp();
        }
        renderer.warp.update(delta_time);
        renderer.metrics.update(delta_time);

        let distance = MOVE_SPEED * delta_time;
        let angle = TURN_SPEED * delta_time;
        camera.move_local(
            axis(&rl, KeyboardKey::KEY_W, KeyboardKey::KEY_S) * distance,
            axis(&rl, KeyboardKey::KEY_D, KeyboardKey::KEY_A) * distance,
            axis(&rl, KeyboardKey::KEY_E, KeyboardKey::KEY_Q) * distance,
        );
        camera.rotate(
            axis(&rl, KeyboardKey::KEY_LEFT, KeyboardKey::KEY_RIGHT) * angle,
            axis(&rl, KeyboardKey::KEY_UP, KeyboardKey::KEY_DOWN) * angle,
            axis(&rl, KeyboardKey::KEY_C, KeyboardKey::KEY_Z) * angle,
        );

        let spin = Mat4::rotation_y(time * SPIN_SPEED);
        let mut objects = vec![RenderObject {
            mesh: &sphere.levels[0],
            lod: Some(&sphere),
            model_matrix: Mat4::scale(Vec3::new(2.0, 2.0, 2.0)) * spin,
            material: sun_material,
            bounding_radius: 2.0,
        }];
        for planet in &planets {
            let center = orbit_point(
                planet.orbit_radius,
                planet.start_angle + time * planet.orbit_speed,
            );
            let size = Vec3::new(planet.radius, planet.radius, planet.radius);
            let group = if planet.material.shader == ShaderKind::Rocky {
                &rock
            } else {
                &sphere
            };
            objects.push(RenderObject {
                mesh: &group.levels[0],
                lod: Some(group),
                model_matrix: Mat4::translation(center) * spin * Mat4::scale(size),
                material: planet.material,
                bounding_radius: planet.radius,
            });
            for segment in (0..ORBIT_SEGMENTS).step_by(2) {
                let step = TAU / ORBIT_SEGMENTS as f32;
                renderer.draw_line_3d(
                    orbit_point(planet.orbit_radius, segment as f32 * step),
                    orbit_point(planet.orbit_radius, (segment + 1) as f32 * step),
                    Color::new(192, 80, 255),
                    false,
                );
            }
        }

        for index in 0..ASTEROID_COUNT {
            let angle = index as f32 / ASTEROID_COUNT as f32 * TAU + time * 0.05;
            let size = 0.3 + 0.25 * (index % 3) as f32;
            objects.push(RenderObject {
                mesh: &rock.levels[0],
                lod: Some(&rock),
                model_matrix: Mat4::translation(orbit_point(ASTEROID_BELT_RADIUS, angle))
                    * Mat4::rotation_y(angle * 3.0)
                    * Mat4::scale(Vec3::new(size, size * 0.7, size)),
                material: asteroid_material,
                bounding_radius: size * 1.5,
            });
        }

        renderer.render(&objects, &camera, time);
        renderer.draw_metrics(Color::new(92, 255, 92));
        renderer.draw_crosshair(Color::new(255, 255, 255));

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
