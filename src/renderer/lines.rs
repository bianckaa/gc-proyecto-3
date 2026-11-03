use super::depth::DepthBuffer;
use super::framebuffer::{Color, Framebuffer};
use super::rasterizer::ScreenVertex;
use crate::engine::math::{Mat4, Vec3, Vec4};

const DASH_ON_PIXELS: i32 = 4;
const DASH_PERIOD_PIXELS: i32 = 7;
const NEAR_W: f32 = 0.001;

fn clip_to_screen(
    a: ScreenVertex,
    b: ScreenVertex,
    width: f32,
    height: f32,
) -> Option<(ScreenVertex, ScreenVertex)> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let mut enter: f32 = 0.0;
    let mut exit: f32 = 1.0;
    let bounds = [
        (-dx, a.x),
        (dx, width - 1.0 - a.x),
        (-dy, a.y),
        (dy, height - 1.0 - a.y),
    ];
    for (direction, distance) in bounds {
        if direction == 0.0 {
            if distance < 0.0 {
                return None;
            }
            continue;
        }
        let t = distance / direction;
        if direction < 0.0 {
            enter = enter.max(t);
        } else {
            exit = exit.min(t);
        }
    }
    if enter > exit {
        return None;
    }
    let at = |t: f32| {
        ScreenVertex::new(
            a.x + dx * t,
            a.y + dy * t,
            a.depth + (b.depth - a.depth) * t,
        )
    };
    Some((at(enter), at(exit)))
}

fn walk_line(a: ScreenVertex, b: ScreenVertex, mut plot: impl FnMut(i32, i32, f32, i32)) {
    let (mut x, mut y) = (a.x.round() as i32, a.y.round() as i32);
    let (end_x, end_y) = (b.x.round() as i32, b.y.round() as i32);
    let dx = (end_x - x).abs();
    let dy = -(end_y - y).abs();
    let step_x = if x < end_x { 1 } else { -1 };
    let step_y = if y < end_y { 1 } else { -1 };
    let total_steps = dx.max(-dy).max(1);
    let mut error = dx + dy;
    let mut step = 0;

    loop {
        let t = step as f32 / total_steps as f32;
        plot(x, y, a.depth + (b.depth - a.depth) * t, step);
        if x == end_x && y == end_y {
            break;
        }
        let doubled_error = 2 * error;
        if doubled_error >= dy {
            error += dy;
            x += step_x;
        }
        if doubled_error <= dx {
            error += dx;
            y += step_y;
        }
        step += 1;
    }
}

fn is_dash_visible(step: i32) -> bool {
    step % DASH_PERIOD_PIXELS < DASH_ON_PIXELS
}

pub fn draw_line_2d(framebuffer: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    let start = ScreenVertex::new(x0 as f32, y0 as f32, 0.0);
    let end = ScreenVertex::new(x1 as f32, y1 as f32, 0.0);
    let width = framebuffer.width() as f32;
    let height = framebuffer.height() as f32;
    if let Some((a, b)) = clip_to_screen(start, end, width, height) {
        walk_line(a, b, |x, y, _, _| framebuffer.set_pixel(x, y, color));
    }
}

fn clip_to_near(a: Vec4, b: Vec4) -> Option<(Vec4, Vec4)> {
    let a_visible = a.w >= NEAR_W;
    let b_visible = b.w >= NEAR_W;
    match (a_visible, b_visible) {
        (true, true) => Some((a, b)),
        (false, false) => None,
        _ => {
            let t = (NEAR_W - a.w) / (b.w - a.w);
            let crossing = a + (b - a) * t;
            if a_visible {
                Some((a, crossing))
            } else {
                Some((crossing, b))
            }
        }
    }
}

pub(super) fn to_screen(clip: Vec4, width: f32, height: f32) -> ScreenVertex {
    let inverse_w = 1.0 / clip.w;
    ScreenVertex::new(
        (clip.x * inverse_w + 1.0) * 0.5 * width,
        (1.0 - clip.y * inverse_w) * 0.5 * height,
        clip.z * inverse_w,
    )
}

pub fn draw_line_3d(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut DepthBuffer,
    view_projection: Mat4,
    start: Vec3,
    end: Vec3,
    color: Color,
    dashed: bool,
) {
    let clip_start = view_projection * start.extend(1.0);
    let clip_end = view_projection * end.extend(1.0);
    let Some((clip_start, clip_end)) = clip_to_near(clip_start, clip_end) else {
        return;
    };

    let width = framebuffer.width() as f32;
    let height = framebuffer.height() as f32;
    let screen_start = to_screen(clip_start, width, height);
    let screen_end = to_screen(clip_end, width, height);
    let Some((a, b)) = clip_to_screen(screen_start, screen_end, width, height) else {
        return;
    };

    walk_line(a, b, |x, y, depth, step| {
        if (!dashed || is_dash_visible(step)) && depth_buffer.test_and_set(x, y, depth) {
            framebuffer.set_pixel(x, y, color);
        }
    });
}
