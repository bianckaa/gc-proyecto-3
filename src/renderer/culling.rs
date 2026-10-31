use super::rasterizer::ScreenVertex;

pub fn is_back_facing(a: ScreenVertex, b: ScreenVertex, c: ScreenVertex) -> bool {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_clockwise_on_screen_faces_the_camera() {
        let a = ScreenVertex::new(0.0, 10.0, 0.0);
        let b = ScreenVertex::new(10.0, 10.0, 0.0);
        let c = ScreenVertex::new(5.0, 0.0, 0.0);
        assert!(!is_back_facing(a, b, c));
        assert!(is_back_facing(a, c, b));
    }
}
