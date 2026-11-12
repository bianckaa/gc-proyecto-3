use gc_proyecto_3::engine::camera::Camera;
use gc_proyecto_3::engine::math::{Mat4, Vec3};
use gc_proyecto_3::game::session::RUN_DURATION_SECONDS;
use gc_proyecto_3::game::sun_lifecycle::{
    black_hole_intensity, blue_shift, collapse_blackout, collapse_contraction, instability,
    instability_offset, radius_multiplier, visual_pulse, FINAL_PHASE_START, FIRST_GROWTH_MARK,
};
use gc_proyecto_3::renderer::framebuffer::{
    Color as FrameColor, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH,
};
use gc_proyecto_3::renderer::lighting::PointLight;
use gc_proyecto_3::renderer::obj::load_normalized;
use gc_proyecto_3::renderer::shaders::ShaderKind;
use gc_proyecto_3::renderer::{Material, RenderObject, Renderer};
use raylib::prelude::*;

const WINDOW_SCALE: i32 = 2;
const SUN_RADIUS: f32 = 1.75;
const ROTATION_SPEED: f32 = 0.24;
const INITIAL_CAMERA_DISTANCE: f32 = 6.2;
const MIN_CAMERA_DISTANCE: f32 = 3.4;
const ROCKY_MIN_CAMERA_DISTANCE: f32 = 4.6;
const MAX_CAMERA_DISTANCE: f32 = 12.0;
const ORBIT_SENSITIVITY: f32 = 0.008;
const ZOOM_STEP: f32 = 0.6;
const MAX_ORBIT_PITCH: f32 = 1.35;
const GAME_SUN_BASE_RADIUS: f32 = 35.0;
const PLANET_OBJ: &str = include_str!("../assets/models/Planet.obj");
const CORE_OBJ: &str = include_str!("../assets/models/CORE_.obj");
const DROID_OBJ: &str = include_str!("../assets/models/Droid_confed.obj");

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlanetSelection {
    Sun,
    Core,
    Rocky,
    Gas,
    Droid,
}

impl PlanetSelection {
    fn label(self) -> &'static str {
        match self {
            Self::Sun => "SOL",
            Self::Core => "CORE",
            Self::Rocky => "ROCOSO",
            Self::Gas => "GASEOSO",
            Self::Droid => "DROID",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SunDebugStage {
    Start,
    MinuteFive,
    MinuteTen,
    BlueTransition,
    Critical,
    Collapse,
    BlackHole,
    FinalBlack,
}

impl SunDebugStage {
    fn next(self) -> Self {
        match self {
            Self::Start => Self::MinuteFive,
            Self::MinuteFive => Self::MinuteTen,
            Self::MinuteTen => Self::BlueTransition,
            Self::BlueTransition => Self::Critical,
            Self::Critical => Self::Collapse,
            Self::Collapse => Self::BlackHole,
            Self::BlackHole => Self::FinalBlack,
            Self::FinalBlack => Self::Start,
        }
    }

    fn elapsed(self) -> f32 {
        match self {
            Self::Start => 0.0,
            Self::MinuteFive => FIRST_GROWTH_MARK,
            Self::MinuteTen => FINAL_PHASE_START,
            Self::BlueTransition => 12.5 * 60.0,
            Self::Critical => 14.5 * 60.0,
            Self::Collapse | Self::BlackHole | Self::FinalBlack => RUN_DURATION_SECONDS,
        }
    }

    fn collapse_progress(self) -> f32 {
        match self {
            Self::Collapse => 0.42,
            Self::BlackHole => 0.82,
            Self::FinalBlack => 1.0,
            _ => 0.0,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Start => "INICIO - 00:00",
            Self::MinuteFive => "CRECIMIENTO 100% - 05:00",
            Self::MinuteTen => "CRECIMIENTO 200% / INESTABLE - 10:00",
            Self::BlueTransition => "TRANSICION AZUL - 12:30",
            Self::Critical => "AZUL CRITICO - 14:30",
            Self::Collapse => "COLAPSO",
            Self::BlackHole => "HOYO NEGRO",
            Self::FinalBlack => "FINAL EN NEGRO",
        }
    }

    fn camera_distance(self) -> f32 {
        let scale =
            radius_multiplier(self.elapsed()) * collapse_contraction(self.collapse_progress());
        (INITIAL_CAMERA_DISTANCE + (scale - 1.0).max(0.0) * 2.4)
            .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE)
    }
}

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
    let start_without_dithering = std::env::args().any(|argument| argument == "--no-dither");
    let (mut rl, thread) = raylib::init()
        .size(
            FRAMEBUFFER_WIDTH as i32 * WINDOW_SCALE,
            FRAMEBUFFER_HEIGHT as i32 * WINDOW_SCALE,
        )
        .title("Planet debug")
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
    let core = load_normalized(CORE_OBJ).expect("no se pudo cargar assets/models/CORE_.obj");
    let droid =
        load_normalized(DROID_OBJ).expect("no se pudo cargar assets/models/Droid_confed.obj");
    let sun_material = Material {
        base_color: Vec3::new(1.0, 0.19, 0.015),
        emissive: Vec3::new(0.08, 0.025, 0.0),
        shader: ShaderKind::Sun,
        outline: true,
        opacity: 1.0,
    };
    let core_material = Material {
        base_color: Vec3::new(0.38, 0.48, 0.54),
        emissive: Vec3::new(0.01, 0.025, 0.03),
        shader: ShaderKind::CorePlanet,
        outline: true,
        opacity: 1.0,
    };
    let rocky_material = Material {
        base_color: Vec3::new(0.40, 0.18, 0.07),
        emissive: Vec3::default(),
        shader: ShaderKind::RockyPlanet,
        outline: true,
        opacity: 1.0,
    };
    let gas_material = Material {
        base_color: Vec3::new(0.20, 0.46, 0.72),
        emissive: Vec3::new(0.0, 0.015, 0.025),
        shader: ShaderKind::GasPlanet,
        outline: true,
        opacity: 1.0,
    };
    let droid_material = Material {
        base_color: Vec3::new(0.46, 0.48, 0.52),
        emissive: Vec3::default(),
        shader: ShaderKind::DroidPlanet,
        outline: true,
        opacity: 1.0,
    };
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, INITIAL_CAMERA_DISTANCE),
        FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32,
    );
    let mut renderer = Renderer::new();
    renderer.dithering_enabled = !start_without_dithering;
    renderer.culling_enabled = false;
    renderer.lod_enabled = false;

    let mut effect_time = 0.0_f32;
    let mut rotation = 0.0_f32;
    let mut rotation_paused = false;
    let mut orbit_yaw = 0.0_f32;
    let mut orbit_pitch = 0.0_f32;
    let mut camera_distance = INITIAL_CAMERA_DISTANCE;
    let requested_planet = std::env::args()
        .skip_while(|argument| argument != "--planet")
        .nth(1)
        .or_else(|| {
            std::env::args()
                .any(|argument| argument == "--rocky")
                .then(|| "rocky".to_owned())
        });
    let mut selection = match requested_planet.as_deref() {
        Some("rocky") => PlanetSelection::Rocky,
        Some("gas") => PlanetSelection::Gas,
        Some("core") => PlanetSelection::Core,
        Some("sun") => PlanetSelection::Sun,
        _ => PlanetSelection::Droid,
    };
    let mut sun_stage = SunDebugStage::Start;
    let mut sun_heat_haze = true;
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
            sun_heat_haze = !sun_heat_haze;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_E) {
            if selection == PlanetSelection::Sun {
                sun_stage = sun_stage.next();
                camera_distance = sun_stage.camera_distance();
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            selection = PlanetSelection::Sun;
            sun_stage = SunDebugStage::Start;
            camera_distance = sun_stage.camera_distance();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            selection = PlanetSelection::Core;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            selection = PlanetSelection::Rocky;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            selection = PlanetSelection::Gas;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FIVE) {
            selection = PlanetSelection::Droid;
        }
        if capture_path.is_none() && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let mouse_delta = rl.get_mouse_delta();
            orbit_yaw -= mouse_delta.x * ORBIT_SENSITIVITY;
            orbit_pitch = (orbit_pitch - mouse_delta.y * ORBIT_SENSITIVITY)
                .clamp(-MAX_ORBIT_PITCH, MAX_ORBIT_PITCH);
        }
        let minimum_camera_distance = if selection == PlanetSelection::Rocky {
            ROCKY_MIN_CAMERA_DISTANCE
        } else {
            MIN_CAMERA_DISTANCE
        };
        if capture_path.is_none() {
            camera_distance = (camera_distance - rl.get_mouse_wheel_move() * ZOOM_STEP)
                .clamp(minimum_camera_distance, MAX_CAMERA_DISTANCE);
        }
        camera_distance = camera_distance.max(minimum_camera_distance);
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            orbit_yaw = 0.0;
            orbit_pitch = 0.0;
            camera_distance = INITIAL_CAMERA_DISTANCE;
        }
        camera.position = orbit_position(orbit_yaw, orbit_pitch, camera_distance);
        camera.look_at(Vec3::default());
        renderer.light = PointLight::new(camera.position, 0.20);
        renderer.heat_haze_enabled = selection == PlanetSelection::Sun && sun_heat_haze;

        let tilt = match selection {
            // El disco del Droid conserva el plano XZ del modelo original.
            PlanetSelection::Droid => Mat4::rotation_x(0.0),
            _ => Mat4::rotation_x(-0.16),
        };
        let (mesh, mut material) = match selection {
            PlanetSelection::Sun => (&sun, sun_material),
            PlanetSelection::Core => (&core, core_material),
            PlanetSelection::Rocky => (&sun, rocky_material),
            PlanetSelection::Gas => (&sun, gas_material),
            PlanetSelection::Droid => (&droid, droid_material),
        };
        let mut object_position = Vec3::default();
        let mut object_radius = SUN_RADIUS;
        let mut blackout = 0.0;
        renderer.black_hole.center = Vec3::default();
        renderer.black_hole.intensity = 0.0;
        if selection == PlanetSelection::Sun {
            let elapsed = sun_stage.elapsed();
            let collapse = sun_stage.collapse_progress();
            let unstable = instability(elapsed);
            let blue = blue_shift(elapsed);
            let blue_star = Vec3::new(0.16, 0.52, 1.0);
            material.base_color = material.base_color * (1.0 - blue) + blue_star * blue;
            material.emissive = material.emissive
                + Vec3::new(0.015, 0.055, 0.18) * unstable
                + Vec3::new(0.0, 0.035, 0.12) * blue;
            object_position =
                instability_offset(elapsed, effect_time) * (SUN_RADIUS / GAME_SUN_BASE_RADIUS);
            object_radius *= radius_multiplier(elapsed)
                * collapse_contraction(collapse)
                * visual_pulse(elapsed, effect_time);
            renderer.black_hole.center = object_position;
            renderer.black_hole.intensity = black_hole_intensity(collapse);
            blackout = collapse_blackout(collapse);
        }
        let model_matrix = Mat4::translation(object_position)
            * tilt
            * Mat4::rotation_y(rotation)
            * Mat4::scale(Vec3::new(object_radius, object_radius, object_radius));
        let objects = [RenderObject {
            mesh,
            model_matrix,
            material,
            bounding_radius: object_radius,
            lod: None,
        }];
        renderer.render(&objects, &camera, effect_time);
        renderer.tint_screen(FrameColor::BLACK, blackout);

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
            if selection == PlanetSelection::Sun {
                drawing.draw_rectangle(8, 8, 690, 54, raylib::color::Color::new(0, 0, 0, 205));
                drawing.draw_text(
                    sun_stage.label(),
                    18,
                    15,
                    18,
                    raylib::color::Color::new(255, 196, 80, 255),
                );
                drawing.draw_text(
                    "E SIGUIENTE ETAPA  |  1-5 PLANETAS  |  H DISTORSION  |  B DITHER",
                    18,
                    39,
                    13,
                    raylib::color::Color::new(220, 220, 230, 255),
                );
            } else {
                drawing.draw_rectangle(8, 8, 180, 36, raylib::color::Color::new(0, 0, 0, 205));
                drawing.draw_text(
                    selection.label(),
                    18,
                    16,
                    20,
                    raylib::color::Color::new(220, 220, 230, 255),
                );
            }
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
