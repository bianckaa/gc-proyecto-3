use gc_proyecto_3::engine::camera::Camera;
use gc_proyecto_3::engine::math::{Mat4, Vec3};
use gc_proyecto_3::renderer::framebuffer::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gc_proyecto_3::renderer::obj::load_normalized;
use gc_proyecto_3::renderer::shaders::ShaderKind;
use gc_proyecto_3::renderer::{Material, RenderObject, Renderer};
use raylib::prelude::*;

const WINDOW_SCALE: i32 = 2;
const SUN_RADIUS: f32 = 1.75;
const ROTATION_SPEED: f32 = 0.24;
const INITIAL_CAMERA_DISTANCE: f32 = 6.2;
const MIN_CAMERA_DISTANCE: f32 = 3.4;
const MAX_CAMERA_DISTANCE: f32 = 12.0;
const ORBIT_SENSITIVITY: f32 = 0.008;
const ZOOM_STEP: f32 = 0.6;
const MAX_ORBIT_PITCH: f32 = 1.35;
const PLANET_OBJ: &str = include_str!("../assets/models/Planet.obj");

fn orbit_position(yaw: f32, pitch: f32, distance: f32) -> Vec3 {
    let horizontal = pitch.cos() * distance;
    Vec3::new(
        yaw.sin() * horizontal,
        pitch.sin() * distance,
        yaw.cos() * horizontal,
    )
}

fn main() {
    let capture_path = std::env::args()
        .skip_while(|argument| argument != "--capture")
        .nth(1);
    let (mut rl, thread) = raylib::init()
        .size(
            FRAMEBUFFER_WIDTH as i32 * WINDOW_SCALE,
            FRAMEBUFFER_HEIGHT as i32 * WINDOW_SCALE,
        )
        .title("Planet debug - Sol")
        .build();
    rl.set_target_fps(60);

    let image = Image::gen_image_color(
        FRAMEBUFFER_WIDTH as i32,
        FRAMEBUFFER_HEIGHT as i32,
        raylib::color::Color::BLACK,
    );
    let mut texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("no se pudo crear la textura del framebuffer");
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

    let sun = load_normalized(PLANET_OBJ).expect("no se pudo cargar assets/models/Planet.obj");
    let material = Material {
        base_color: Vec3::new(1.0, 0.19, 0.015),
        emissive: Vec3::new(0.08, 0.025, 0.0),
        shader: ShaderKind::Sun,
        outline: true,
    };
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, INITIAL_CAMERA_DISTANCE),
        FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32,
    );
    let mut renderer = Renderer::new();
    renderer.heat_haze_enabled = true;
    renderer.culling_enabled = false;
    renderer.lod_enabled = false;

    let mut effect_time = 0.0_f32;
    let mut rotation = 0.0_f32;
    let mut rotation_paused = false;
    let mut orbit_yaw = 0.0_f32;
    let mut orbit_pitch = 0.0_f32;
    let mut camera_distance = INITIAL_CAMERA_DISTANCE;
    let mut rendered_frames = 0_u32;

    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time().min(0.05);
        effect_time += delta_time;
        if !rotation_paused {
            rotation += delta_time * ROTATION_SPEED;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            rotation_paused = !rotation_paused;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            renderer.dithering_enabled = !renderer.dithering_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_O) {
            renderer.outline_enabled = !renderer.outline_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            renderer.heat_haze_enabled = !renderer.heat_haze_enabled;
        }
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let mouse_delta = rl.get_mouse_delta();
            orbit_yaw -= mouse_delta.x * ORBIT_SENSITIVITY;
            orbit_pitch = (orbit_pitch - mouse_delta.y * ORBIT_SENSITIVITY)
                .clamp(-MAX_ORBIT_PITCH, MAX_ORBIT_PITCH);
        }
        camera_distance = (camera_distance - rl.get_mouse_wheel_move() * ZOOM_STEP)
            .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            orbit_yaw = 0.0;
            orbit_pitch = 0.0;
            camera_distance = INITIAL_CAMERA_DISTANCE;
        }
        camera.position = orbit_position(orbit_yaw, orbit_pitch, camera_distance);
        camera.look_at(Vec3::default());

        let model_matrix = Mat4::rotation_x(-0.16)
            * Mat4::rotation_y(rotation)
            * Mat4::scale(Vec3::new(SUN_RADIUS, SUN_RADIUS, SUN_RADIUS));
        let objects = [RenderObject {
            mesh: &sun,
            model_matrix,
            material,
            bounding_radius: SUN_RADIUS,
            lod: None,
        }];
        renderer.render(&objects, &camera, effect_time);

        texture
            .update_texture(renderer.pixels())
            .expect("no se pudo actualizar la textura del framebuffer");
        {
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
        rendered_frames += 1;
        if rendered_frames == 90 {
            if let Some(path) = &capture_path {
                rl.take_screenshot(&thread, path);
                break;
            }
        }
    }
}
