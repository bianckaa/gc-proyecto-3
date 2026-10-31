use gc_proyecto_3::engine::camera::Camera;
use gc_proyecto_3::engine::math::{Mat4, Vec2, Vec3};
use gc_proyecto_3::renderer::depth::DepthBuffer;
use gc_proyecto_3::renderer::framebuffer::{Color, Framebuffer, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gc_proyecto_3::renderer::lines::{draw_line_2d, draw_line_3d};
use gc_proyecto_3::renderer::pipeline::draw_mesh;
use gc_proyecto_3::renderer::rasterizer::Fragment;
use gc_proyecto_3::renderer::{Mesh, Vertex};
use raylib::prelude::*;

const WINDOW_SCALE: i32 = 2;
const SPIN_SPEED: f32 = 1.0;
const SHIP_ORBIT_RADIUS: f32 = 10.0;
const SHIP_ORBIT_SPEED: f32 = 0.5;
const SHIP_HEIGHT: f32 = 2.0;
const FLOOR_SIZE: f32 = 30.0;
const FLOOR_Y: f32 = -1.5;
const FLOOR_TILES: f32 = 16.0;
const MOVE_SPEED: f32 = 8.0;
const TURN_SPEED: f32 = 1.5;
const FOLLOW_DISTANCE: f32 = 5.0;
const FOLLOW_HEIGHT: f32 = 2.0;
const FOLLOW_LOOK_AHEAD: f32 = 6.0;
const FOLLOW_SMOOTHING: f32 = 6.0;

fn push_vertices(mesh: &mut Mesh, positions: &[Vec3], uvs: &[Vec2], indices: &[u32]) {
    let edge_a = positions[1] - positions[0];
    let edge_b = positions[2] - positions[0];
    let normal = edge_a.cross(edge_b).normalize();
    let base = mesh.vertices.len() as u32;
    for (&position, &uv) in positions.iter().zip(uvs) {
        mesh.vertices.push(Vertex { position, normal, uv });
    }
    mesh.indices.extend(indices.iter().map(|index| base + index));
}

fn push_quad(mesh: &mut Mesh, corners: [Vec3; 4], tiles: f32) {
    let uvs = [
        Vec2::new(0.0, 0.0),
        Vec2::new(tiles, 0.0),
        Vec2::new(tiles, tiles),
        Vec2::new(0.0, tiles),
    ];
    push_vertices(mesh, &corners, &uvs, &[0, 1, 2, 0, 2, 3]);
}

fn cube_mesh() -> Mesh {
    let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    let corners = [
        p(-1.0, -1.0, -1.0),
        p(1.0, -1.0, -1.0),
        p(1.0, 1.0, -1.0),
        p(-1.0, 1.0, -1.0),
        p(-1.0, -1.0, 1.0),
        p(1.0, -1.0, 1.0),
        p(1.0, 1.0, 1.0),
        p(-1.0, 1.0, 1.0),
    ];
    let faces = [
        [4, 5, 6, 7],
        [1, 0, 3, 2],
        [5, 1, 2, 6],
        [0, 4, 7, 3],
        [7, 6, 2, 3],
        [0, 1, 5, 4],
    ];
    let mut mesh = Mesh::default();
    for face in faces {
        push_quad(&mut mesh, face.map(|index: usize| corners[index]), 1.0);
    }
    mesh
}

fn floor_mesh() -> Mesh {
    let mut mesh = Mesh::default();
    let s = FLOOR_SIZE;
    push_quad(
        &mut mesh,
        [
            Vec3::new(-s, FLOOR_Y, s),
            Vec3::new(s, FLOOR_Y, s),
            Vec3::new(s, FLOOR_Y, -s),
            Vec3::new(-s, FLOOR_Y, -s),
        ],
        FLOOR_TILES,
    );
    mesh
}

fn ship_mesh() -> Mesh {
    let nose = Vec3::new(0.0, 0.0, -1.2);
    let left = Vec3::new(-1.0, 0.0, 0.3);
    let right = Vec3::new(1.0, 0.0, 0.3);
    let top = Vec3::new(0.0, 0.25, 0.1);
    let bottom = Vec3::new(0.0, -0.15, 0.1);
    let faces = [
        [nose, left, top],
        [nose, top, right],
        [left, right, top],
        [nose, bottom, left],
        [nose, right, bottom],
        [right, left, bottom],
    ];
    let mut mesh = Mesh::default();
    for face in faces {
        push_vertices(&mut mesh, &face, &[Vec2::default(); 3], &[0, 1, 2]);
    }
    mesh
}

fn shade_checker(fragment: &Fragment) -> Color {
    let cell = fragment.uv.x.floor() as i32 + fragment.uv.y.floor() as i32;
    if cell.rem_euclid(2) == 0 {
        Color::new(192, 80, 255)
    } else {
        Color::new(40, 16, 56)
    }
}

fn shade_normal(fragment: &Fragment) -> Color {
    let channel = |value: f32| ((value * 0.5 + 0.5) * 255.0) as u8;
    let n = fragment.normal;
    Color::new(channel(n.x), channel(n.y), channel(n.z))
}

fn shade_ship(fragment: &Fragment) -> Color {
    let light = 0.4 + 0.6 * fragment.normal.y.abs();
    Color::new((92.0 * light) as u8, (255.0 * light) as u8, (92.0 * light) as u8)
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

    let mut framebuffer = Framebuffer::new();
    let mut depth_buffer = DepthBuffer::new();
    let cube = cube_mesh();
    let floor = floor_mesh();
    let ship = ship_mesh();

    let aspect = FRAMEBUFFER_WIDTH as f32 / FRAMEBUFFER_HEIGHT as f32;
    let mut camera = Camera::new(Vec3::new(0.0, 3.0, 16.0), aspect);
    let mut following = true;
    let mut spin = 0.0_f32;
    let mut ship_angle = 0.0_f32;

    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        spin += SPIN_SPEED * delta_time;
        ship_angle += SHIP_ORBIT_SPEED * delta_time;

        let (sin, cos) = ship_angle.sin_cos();
        let ship_position = Vec3::new(SHIP_ORBIT_RADIUS * cos, SHIP_HEIGHT, SHIP_ORBIT_RADIUS * sin);
        let ship_forward = Vec3::new(-sin, 0.0, cos);
        let ship_up = Vec3::new(0.0, 1.0, 0.0);

        if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            following = !following;
        }
        if following {
            camera.follow(
                ship_position,
                ship_forward,
                ship_up,
                FOLLOW_DISTANCE,
                FOLLOW_HEIGHT,
                FOLLOW_LOOK_AHEAD,
                FOLLOW_SMOOTHING,
                delta_time,
            );
        } else {
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
        }

        framebuffer.clear(Color::BLACK);
        depth_buffer.clear();
        let view_projection = camera.view_projection_matrix();

        draw_mesh(
            &mut framebuffer,
            &mut depth_buffer,
            &floor,
            Mat4::identity(),
            view_projection,
            shade_checker,
        );
        draw_mesh(
            &mut framebuffer,
            &mut depth_buffer,
            &cube,
            Mat4::rotation_y(spin) * Mat4::rotation_x(spin * 0.6),
            view_projection,
            shade_normal,
        );
        draw_mesh(
            &mut framebuffer,
            &mut depth_buffer,
            &ship,
            Mat4::translation(ship_position)
                * Mat4::rotation_y(std::f32::consts::PI - ship_angle),
            view_projection,
            shade_ship,
        );
        draw_line_3d(
            &mut framebuffer,
            &mut depth_buffer,
            view_projection,
            ship_position,
            ship_position + ship_forward * 20.0,
            Color::new(255, 48, 64),
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
