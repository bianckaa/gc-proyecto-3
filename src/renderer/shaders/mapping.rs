use super::*;

pub(super) fn core_projection(point: Vec3) -> Vec2 {
    let direction = point.normalize();
    let (ax, ay, az) = (direction.x.abs(), direction.y.abs(), direction.z.abs());
    if ax >= ay && ax >= az {
        Vec2::new(direction.z, direction.y)
    } else if ay >= az {
        Vec2::new(direction.x, direction.z)
    } else {
        Vec2::new(direction.x, direction.y)
    }
}

pub(super) fn panel_sample(uv: Vec2, columns: f32, rows: f32, seed: i32) -> (f32, f32, Vec2) {
    let scaled = Vec2::new(uv.x * columns, uv.y * rows);
    let cell_x = scaled.x.floor() as i32;
    let cell_y = scaled.y.floor() as i32;
    let local = Vec2::new(scaled.x.rem_euclid(1.0), scaled.y.rem_euclid(1.0));
    let value = hash(cell_x, cell_y, seed);
    let edge = local.x.min(1.0 - local.x).min(local.y.min(1.0 - local.y));
    (value, edge, local)
}
