use gc_proyecto_3::engine::math::{Mat4, Vec3};
use gc_proyecto_3::renderer::depth::DepthBuffer;
use gc_proyecto_3::renderer::framebuffer::{Color, Framebuffer, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gc_proyecto_3::renderer::lines::{draw_line_2d, draw_line_3d};
use gc_proyecto_3::renderer::rasterizer::{fill_triangle, ScreenVertex};
use raylib::prelude::*;

const WINDOW_SCALE: i32 = 3;
const NEAR_PLANE: f32 = 0.1;
const FAR_PLANE: f32 = 100.0;
const SPIN_SPEED_Y: f32 = 1.0;
const SPIN_SPEED_X: f32 = 0.6;

const CUBE_CORNERS: [Vec3; 8] = [
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
];

const CUBE_FACES: [([usize; 4], Color); 6] = [
    ([4, 5, 6, 7], Color::new(92, 255, 92)),
    ([1, 0, 3, 2], Color::new(192, 80, 255)),
    ([5, 1, 2, 6], Color::new(217, 194, 154)),
    ([0, 4, 7, 3], Color::new(255, 48, 64)),
    ([7, 6, 2, 3], Color::new(80, 180, 255)),
    ([0, 1, 5, 4], Color::new(255, 200, 60)),
];

#[derive(Clone, Copy, PartialEq)]
enum Scene {
    Triangle,
    Cube,
}

fn project(view_projection: Mat4, model: Mat4, point: Vec3) -> Option<ScreenVertex> {
    let clip = view_projection * model * point.extend(1.0);
    if clip.w < NEAR_PLANE {
        return None;
    }
    let inverse_w = 1.0 / clip.w;
    Some(ScreenVertex::new(
        (clip.x * inverse_w + 1.0) * 0.5 * FRAMEBUFFER_WIDTH as f32,
        (1.0 - clip.y * inverse_w) * 0.5 * FRAMEBUFFER_HEIGHT as f32,
        clip.z * inverse_w,
    ))
}

fn draw_world_triangle(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    view_projection: Mat4,
    model: Mat4,
    corners: [Vec3; 3],
    color: Color,
) {
    let projected = corners.map(|corner| project(view_projection, model, corner));
    if let [Some(a), Some(b), Some(c)] = projected {
        fill_triangle(framebuffer, depth_buffer, a, b, c, color);
    }
}

fn draw_cube(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    view_projection: Mat4,
    model: Mat4,
) {
    for (face, color) in CUBE_FACES {
        let [a, b, c, d] = face.map(|index| CUBE_CORNERS[index]);
        draw_world_triangle(framebuffer, depth_buffer, view_projection, model, [a, b, c], color);
        draw_world_triangle(framebuffer, depth_buffer, view_projection, model, [a, c, d], color);
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

    let mut framebuffer = Framebuffer::new();
    let mut depth_buffer = DepthBuffer::new();
    let mut scene = Scene::Cube;
    let mut angle_y = 0.0_f32;
    let mut angle_x = 0.0_f32;

    let aspect = FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32;
    let projection = Mat4::perspective(60.0_f32.to_radians(), aspect, NEAR_PLANE, FAR_PLANE);
    let view = Mat4::look_at(
        Vec3::new(0.0, 1.5, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let view_projection = projection * view;

    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        angle_y += SPIN_SPEED_Y * delta_time;
        angle_x += SPIN_SPEED_X * delta_time;

        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            scene = Scene::Triangle;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            scene = Scene::Cube;
        }

        framebuffer.clear(Color::BLACK);
        depth_buffer.clear();

        let model = Mat4::rotation_y(angle_y) * Mat4::rotation_x(angle_x);
        match scene {
            Scene::Triangle => draw_world_triangle(
                &mut framebuffer,
                &mut depth_buffer,
                view_projection,
                Mat4::rotation_y(angle_y),
                [
                    Vec3::new(-1.5, -1.0, 0.0),
                    Vec3::new(1.5, -1.0, 0.0),
                    Vec3::new(0.0, 1.5, 0.0),
                ],
                Color::new(92, 255, 92),
            ),
            Scene::Cube => draw_cube(&mut framebuffer, &mut depth_buffer, view_projection, model),
        }

        draw_line_3d(
            &mut framebuffer,
            &mut depth_buffer,
            view_projection,
            Vec3::new(-3.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Color::new(255, 48, 64),
            false,
        );
        draw_line_3d(
            &mut framebuffer,
            &mut depth_buffer,
            view_projection,
            Vec3::new(0.0, 0.0, -3.0),
            Vec3::new(0.0, 0.0, 3.0),
            Color::new(192, 80, 255),
            true,
        );

        let center_x = FRAMEBUFFER_WIDTH as i32 / 2;
        let center_y = FRAMEBUFFER_HEIGHT as i32 / 2;
        let white = Color::new(255, 255, 255);
        draw_line_2d(&mut framebuffer, center_x - 4, center_y, center_x + 4, center_y, white);
        draw_line_2d(&mut framebuffer, center_x, center_y - 4, center_x, center_y + 4, white);

        texture
            .update_texture(framebuffer.pixels())
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
