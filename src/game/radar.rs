use crate::engine::math::{Mat4, Vec3};
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_WIDTH};
use crate::renderer::Renderer;

const SIZE: i32 = 80;
const LEFT: i32 = FRAMEBUFFER_WIDTH as i32 - SIZE - 6;
const TOP: i32 = 18;
const RANGE: f32 = 160.0;
const SCREEN_SCALE: f32 = SIZE as f32 * 0.47;
const CAMERA_POSITION: Vec3 = Vec3::new(0.0, 2.4, -4.8);
const FRAME: Color = Color::new(105, 125, 150);
const GRID: Color = Color::new(38, 55, 70);
const PLAYER: Color = Color::new(90, 255, 130);
const ENEMY_FRONT: Color = Color::new(255, 64, 64);
const ENEMY_BEHIND: Color = Color::new(205, 80, 255);
const ARRIVING: Color = Color::new(255, 210, 80);

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
const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];
const CENTER_PLANE: [Vec3; 4] = [
    Vec3::new(-1.0, 0.0, -1.0),
    Vec3::new(1.0, 0.0, -1.0),
    Vec3::new(1.0, 0.0, 1.0),
    Vec3::new(-1.0, 0.0, 1.0),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarContact {
    pub position: Vec3,
    pub arriving: bool,
}

pub fn draw(
    renderer: &mut Renderer,
    ship_position: Vec3,
    ship_right: Vec3,
    ship_up: Vec3,
    ship_forward: Vec3,
    contacts: impl IntoIterator<Item = RadarContact>,
) {
    renderer.draw_rect_2d(LEFT, TOP, SIZE, SIZE, Color::BLACK);
    draw_volume(renderer);
    for contact in contacts {
        let local = local_contact(
            ship_position,
            ship_right,
            ship_up,
            ship_forward,
            contact.position,
        );
        let Some((x, y)) = project(local) else {
            continue;
        };
        if local.y.abs() > 0.04 {
            if let Some((ground_x, ground_y)) = project(Vec3::new(local.x, 0.0, local.z)) {
                renderer.draw_line_2d(x, y, ground_x, ground_y, GRID);
            }
        }
        let color = if contact.arriving {
            ARRIVING
        } else if local.z >= 0.0 {
            ENEMY_FRONT
        } else {
            ENEMY_BEHIND
        };
        renderer.draw_rect_2d(x - 1, y - 1, 3, 3, color);
    }
    draw_player(renderer);
}

fn draw_volume(renderer: &mut Renderer) {
    for (start, end) in CUBE_EDGES {
        if let (Some(a), Some(b)) = (project(CUBE_CORNERS[start]), project(CUBE_CORNERS[end])) {
            renderer.draw_line_2d(a.0, a.1, b.0, b.1, FRAME);
        }
    }
    for index in 0..CENTER_PLANE.len() {
        let next = (index + 1) % CENTER_PLANE.len();
        if let (Some(a), Some(b)) = (project(CENTER_PLANE[index]), project(CENTER_PLANE[next])) {
            renderer.draw_line_2d(a.0, a.1, b.0, b.1, GRID);
        }
    }
}

fn draw_player(renderer: &mut Renderer) {
    let Some(center) = project(Vec3::default()) else {
        return;
    };
    let Some(forward) = project(Vec3::new(0.0, 0.0, 0.65)) else {
        return;
    };
    renderer.draw_line_2d(center.0, center.1 - 3, center.0 + 3, center.1, PLAYER);
    renderer.draw_line_2d(center.0 + 3, center.1, center.0, center.1 + 3, PLAYER);
    renderer.draw_line_2d(center.0, center.1 + 3, center.0 - 3, center.1, PLAYER);
    renderer.draw_line_2d(center.0 - 3, center.1, center.0, center.1 - 3, PLAYER);
    renderer.draw_line_2d(center.0, center.1, forward.0, forward.1, PLAYER);
}

fn local_contact(
    ship_position: Vec3,
    ship_right: Vec3,
    ship_up: Vec3,
    ship_forward: Vec3,
    enemy_position: Vec3,
) -> Vec3 {
    let offset = enemy_position - ship_position;
    let mut local = Vec3::new(
        offset.dot(ship_right) / RANGE,
        offset.dot(ship_up) / RANGE,
        offset.dot(ship_forward) / RANGE,
    );
    let edge = local.x.abs().max(local.y.abs()).max(local.z.abs());
    if edge > 1.0 {
        local = local * (1.0 / edge);
    }
    local
}

fn projection() -> Mat4 {
    let view = Mat4::look_at(CAMERA_POSITION, Vec3::default(), Vec3::new(0.0, 1.0, 0.0));
    let projection = Mat4::perspective(50.0_f32.to_radians(), 1.0, 0.1, 20.0);
    projection * view
}

fn project(point: Vec3) -> Option<(i32, i32)> {
    let clip = projection() * point.extend(1.0);
    if clip.w <= 0.001 {
        return None;
    }
    let center_x = LEFT as f32 + SIZE as f32 * 0.5;
    let center_y = TOP as f32 + SIZE as f32 * 0.5;
    Some((
        (center_x - clip.x / clip.w * SCREEN_SCALE).round() as i32,
        (center_y - clip.y / clip.w * SCREEN_SCALE).round() as i32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_contacts_follow_all_three_ship_axes_and_clamp_to_range() {
        let position = Vec3::default();
        let right = Vec3::new(1.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, -1.0);

        assert_eq!(
            local_contact(position, right, up, forward, Vec3::new(80.0, 40.0, -120.0)),
            Vec3::new(0.5, 0.25, 0.75)
        );
        let distant = local_contact(position, right, up, forward, Vec3::new(800.0, 0.0, 0.0));
        assert_eq!(distant, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn virtual_radar_camera_keeps_player_centered_and_separates_depth_and_height() {
        let center = project(Vec3::default()).unwrap();
        let above = project(Vec3::new(0.0, 1.0, 0.0)).unwrap();
        let starboard = project(Vec3::new(1.0, 0.0, 0.0)).unwrap();
        let front = project(Vec3::new(0.0, 0.0, 1.0)).unwrap();
        let behind = project(Vec3::new(0.0, 0.0, -1.0)).unwrap();

        assert_eq!(center, (LEFT + SIZE / 2, TOP + SIZE / 2));
        assert!(above.1 < center.1);
        assert!(starboard.0 > center.0);
        assert_ne!(front, behind);
        for corner in CUBE_CORNERS {
            let point = project(corner).unwrap();
            assert!(point.0 >= LEFT && point.0 <= LEFT + SIZE);
            assert!(point.1 >= TOP && point.1 <= TOP + SIZE);
        }
    }

    #[test]
    fn radar_draws_player_front_rear_and_arriving_contacts() {
        let mut renderer = Renderer::new();
        let contacts = [
            RadarContact {
                position: Vec3::new(40.0, 0.0, -40.0),
                arriving: false,
            },
            RadarContact {
                position: Vec3::new(-40.0, 0.0, 40.0),
                arriving: false,
            },
            RadarContact {
                position: Vec3::new(0.0, 50.0, -60.0),
                arriving: true,
            },
        ];
        draw(
            &mut renderer,
            Vec3::default(),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            contacts,
        );
        let contains = |color: Color| {
            renderer
                .pixels()
                .chunks_exact(4)
                .any(|pixel| pixel[..3] == [color.r, color.g, color.b])
        };

        assert!(contains(PLAYER));
        assert!(contains(ENEMY_FRONT));
        assert!(contains(ENEMY_BEHIND));
        assert!(contains(ARRIVING));
    }
}
