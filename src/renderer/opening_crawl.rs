use super::dithering::threshold;
use super::framebuffer::{Color, Framebuffer};
use super::pixel_font::{for_each_cell, text_width, GLYPH_HEIGHT};
use super::rasterizer::{for_each_covered_pixel, ScreenVertex};
use crate::engine::camera::Camera;
use crate::engine::math::{Mat4, Vec3};
use std::collections::HashSet;

pub const PRELUDE: [&str; 2] = [
    "HACE NO MUCHO TIEMPO, EN UN LABORATORIO",
    "DE GRÁFICAS MUY, MUY CERCANO....",
];

pub const TITLE: [&str; 2] = ["SISTEMA", "SOLAR"];

pub const CRAWL: [&str; 23] = [
    "PROYECTO III",
    "EL RENDERER DE SOFTWARE",
    "",
    "ES UNA ÉPOCA OSCURA PARA LA",
    "GALAXIA. LAS TARJETAS DE VIDEO",
    "HAN SIDO PROHIBIDAS Y CADA",
    "PÍXEL DEBE CALCULARSE A MANO.",
    "",
    "UN PEQUEÑO EQUIPO DE REBELDES",
    "HA CONSTRUIDO UN MOTOR GRÁFICO",
    "DESDE CERO: MATRICES, Z-BUFFER,",
    "SHADERS Y MILES DE TRIÁNGULOS",
    "QUE GIRAN ALREDEDOR DE UN SOL",
    "ARDIENTE.",
    "",
    "AHORA, A BORDO DE LA NAVE ETA,",
    "UN PILOTO SOLITARIO SE PREPARA",
    "PARA RECORRER EL SISTEMA SOLAR",
    "Y DEMOSTRAR QUE NO SE NECESITA",
    "UNA GPU PARA ALCANZAR LAS",
    "ESTRELLAS....",
    "",
    "",
];

const PRELUDE_COLOR: Color = Color::new(75, 213, 238);
const GOLD: Color = Color::new(229, 177, 58);

const PRELUDE_START: f32 = 1.0;
const PRELUDE_END: f32 = 7.0;
const PRELUDE_FADE: f32 = 1.0;
const PRELUDE_LINE_GAP: i32 = 12;

const TITLE_START: f32 = 7.5;
const TITLE_DURATION: f32 = 8.0;
const TITLE_FADE: f32 = 2.5;
const TITLE_START_CELL: f32 = 9.0;
const TITLE_SHRINK_RATE: f32 = 0.45;
const TITLE_LINE_ROWS: f32 = 9.0;

const CRAWL_START: f32 = 9.0;
const CRAWL_SPEED: f32 = 0.3;
const CRAWL_EYE_HEIGHT: f32 = 1.0;
const CRAWL_LOOK_DISTANCE: f32 = 6.0;
const CRAWL_FOV_DEGREES: f32 = 50.0;
const CRAWL_ENTRY_DISTANCE: f32 = 1.3;
const CRAWL_CELL_WIDTH: f32 = 0.012;
const CRAWL_CELL_DEPTH: f32 = 0.03;
const CRAWL_LINE_SPACING: f32 = 0.3;
const CRAWL_FADE_START: f32 = 5.0;
const CRAWL_FADE_END: f32 = 9.0;
const NEAR_PLANE: f32 = 0.05;
const FAR_PLANE: f32 = 100.0;

const PAN_DURATION: f32 = 4.0;
const PAN_LOOK_UP: f32 = 1.6;

pub struct OpeningCrawl {
    elapsed: f32,
    skipped_at: Option<f32>,
}

fn smoothstep(edge_start: f32, edge_end: f32, value: f32) -> f32 {
    let t = ((value - edge_start) / (edge_end - edge_start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn fade_window(time: f32, start: f32, end: f32, fade: f32) -> f32 {
    smoothstep(start, start + fade, time) * (1.0 - smoothstep(end - fade, end, time))
}

fn plot(framebuffer: &mut Framebuffer, x: i32, y: i32, color: Color, intensity: f32) {
    if intensity > threshold(x, y) {
        framebuffer.set_pixel(x, y, color);
    }
}

fn crawl_end() -> f32 {
    let travel = CRAWL.len() as f32 * CRAWL_LINE_SPACING + CRAWL_FADE_END;
    CRAWL_START + travel / CRAWL_SPEED
}

fn crawl_projection(framebuffer: &Framebuffer) -> Mat4 {
    let aspect = framebuffer.width() as f32 / framebuffer.height() as f32;
    let projection = Mat4::perspective(
        CRAWL_FOV_DEGREES.to_radians(),
        aspect,
        NEAR_PLANE,
        FAR_PLANE,
    );
    let view = Mat4::look_at(
        Vec3::new(0.0, CRAWL_EYE_HEIGHT, 0.0),
        Vec3::new(0.0, 0.0, -CRAWL_LOOK_DISTANCE),
        Vec3::new(0.0, 1.0, 0.0),
    );
    projection * view
}

fn project(matrix: Mat4, point: Vec3, width: f32, height: f32) -> Option<ScreenVertex> {
    let clip = matrix * point.extend(1.0);
    if clip.w < NEAR_PLANE {
        return None;
    }
    Some(ScreenVertex::new(
        (clip.x / clip.w + 1.0) * 0.5 * width,
        (1.0 - clip.y / clip.w) * 0.5 * height,
        clip.z / clip.w,
    ))
}

impl OpeningCrawl {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            skipped_at: None,
        }
    }

    pub fn update(&mut self, delta_time: f32, skip: bool) {
        self.elapsed += delta_time;
        if skip && self.skipped_at.is_none() && self.elapsed < self.pan_start() {
            self.skipped_at = Some(self.elapsed);
        }
    }

    fn pan_start(&self) -> f32 {
        self.skipped_at.unwrap_or_else(crawl_end)
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.pan_start() + PAN_DURATION
    }

    pub fn pan_progress(&self) -> f32 {
        smoothstep(self.pan_start(), self.pan_start() + PAN_DURATION, self.elapsed)
    }

    pub fn camera(&self, game_camera: &Camera) -> Camera {
        let forward = game_camera.forward();
        let looking_up = (forward + Vec3::new(0.0, PAN_LOOK_UP, 0.0)).normalize();
        let progress = self.pan_progress();
        let direction = (looking_up + (forward - looking_up) * progress).normalize();
        let mut camera = *game_camera;
        camera.look_at(camera.position + direction);
        camera
    }

    pub fn draw(&self, framebuffer: &mut Framebuffer) {
        if self.elapsed >= self.pan_start() {
            return;
        }
        self.draw_prelude(framebuffer);
        self.draw_title(framebuffer);
        self.draw_crawl(framebuffer);
    }

    fn draw_prelude(&self, framebuffer: &mut Framebuffer) {
        let intensity = fade_window(self.elapsed, PRELUDE_START, PRELUDE_END, PRELUDE_FADE);
        if intensity <= 0.0 {
            return;
        }
        let center_x = framebuffer.width() as i32 / 2;
        let top = framebuffer.height() as i32 / 2 - PRELUDE_LINE_GAP;
        for (index, line) in PRELUDE.iter().enumerate() {
            let left = center_x - text_width(line) / 2;
            let y = top + index as i32 * PRELUDE_LINE_GAP;
            for_each_cell(line, |column, row| {
                plot(framebuffer, left + column, y + row, PRELUDE_COLOR, intensity);
            });
        }
    }

    fn draw_title(&self, framebuffer: &mut Framebuffer) {
        let time = self.elapsed - TITLE_START;
        if !(0.0..TITLE_DURATION).contains(&time) {
            return;
        }
        let cell = TITLE_START_CELL * (-TITLE_SHRINK_RATE * time).exp();
        let intensity = 1.0 - smoothstep(TITLE_DURATION - TITLE_FADE, TITLE_DURATION, time);
        let center_x = framebuffer.width() as f32 * 0.5;
        let center_y = framebuffer.height() as f32 * 0.5;
        let block_height = TITLE.len() as f32 * TITLE_LINE_ROWS - 2.0;
        for (index, line) in TITLE.iter().enumerate() {
            let mut cells = HashSet::new();
            for_each_cell(line, |column, row| {
                cells.insert((column, row));
            });
            let left = center_x - text_width(line) as f32 * cell * 0.5;
            let top = center_y + (index as f32 * TITLE_LINE_ROWS - block_height * 0.5) * cell;
            for &(column, row) in &cells {
                let x0 = (left + column as f32 * cell).round() as i32;
                let y0 = (top + row as f32 * cell).round() as i32;
                let x1 = (left + (column + 1) as f32 * cell).round() as i32;
                let y1 = (top + (row + 1) as f32 * cell).round() as i32;
                let open = |dx: i32, dy: i32| !cells.contains(&(column + dx, row + dy));
                if open(0, -1) {
                    (x0..=x1).for_each(|x| plot(framebuffer, x, y0, GOLD, intensity));
                }
                if open(0, 1) {
                    (x0..=x1).for_each(|x| plot(framebuffer, x, y1, GOLD, intensity));
                }
                if open(-1, 0) {
                    (y0..=y1).for_each(|y| plot(framebuffer, x0, y, GOLD, intensity));
                }
                if open(1, 0) {
                    (y0..=y1).for_each(|y| plot(framebuffer, x1, y, GOLD, intensity));
                }
            }
        }
    }

    fn draw_crawl(&self, framebuffer: &mut Framebuffer) {
        let scroll = (self.elapsed - CRAWL_START) * CRAWL_SPEED;
        if scroll <= 0.0 {
            return;
        }
        let matrix = crawl_projection(framebuffer);
        let (width, height) = (framebuffer.width() as f32, framebuffer.height() as f32);
        for (index, line) in CRAWL.iter().enumerate() {
            let baseline = CRAWL_ENTRY_DISTANCE + scroll - index as f32 * CRAWL_LINE_SPACING;
            let farthest = baseline + GLYPH_HEIGHT as f32 * CRAWL_CELL_DEPTH;
            if farthest < CRAWL_ENTRY_DISTANCE * 0.5 || baseline > CRAWL_FADE_END {
                continue;
            }
            let left = -text_width(line) as f32 * CRAWL_CELL_WIDTH * 0.5;
            for_each_cell(line, |column, row| {
                let near = baseline + (GLYPH_HEIGHT - 1 - row) as f32 * CRAWL_CELL_DEPTH;
                let far = near + CRAWL_CELL_DEPTH;
                let intensity = 1.0 - smoothstep(CRAWL_FADE_START, CRAWL_FADE_END, near);
                let x0 = left + column as f32 * CRAWL_CELL_WIDTH;
                let x1 = x0 + CRAWL_CELL_WIDTH;
                let corners = [
                    Vec3::new(x0, 0.0, -near),
                    Vec3::new(x1, 0.0, -near),
                    Vec3::new(x1, 0.0, -far),
                    Vec3::new(x0, 0.0, -far),
                ]
                .map(|corner| project(matrix, corner, width, height));
                let [Some(a), Some(b), Some(c), Some(d)] = corners else {
                    return;
                };
                let (frame_width, frame_height) = (framebuffer.width(), framebuffer.height());
                for [p, q, r] in [[a, b, c], [a, c, d]] {
                    for_each_covered_pixel(frame_width, frame_height, p, q, r, |x, y, _| {
                        plot(framebuffer, x, y, GOLD, intensity);
                    });
                }
            });
        }
    }
}

impl Default for OpeningCrawl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::pixel_font::supports;

    fn lit_pixels(framebuffer: &Framebuffer) -> usize {
        framebuffer
            .pixels()
            .chunks(4)
            .filter(|pixel| pixel[..3] != [0, 0, 0])
            .count()
    }

    fn frame_at(seconds: f32) -> usize {
        let mut opening = OpeningCrawl::new();
        opening.update(seconds, false);
        let mut framebuffer = Framebuffer::new();
        opening.draw(&mut framebuffer);
        lit_pixels(&framebuffer)
    }

    #[test]
    fn every_line_uses_supported_characters() {
        for line in PRELUDE.iter().chain(TITLE.iter()).chain(CRAWL.iter()) {
            assert!(line.chars().all(supports), "caracter no soportado en: {line}");
        }
    }

    #[test]
    fn each_stage_draws_something() {
        assert_eq!(frame_at(0.2), 0);
        assert!(frame_at(4.0) > 0);
        assert!(frame_at(8.0) > 0);
        assert!(frame_at(30.0) > 0);
    }

    #[test]
    fn skipping_starts_the_pan_and_then_finishes() {
        let mut opening = OpeningCrawl::new();
        opening.update(3.0, true);
        assert_eq!(opening.pan_progress(), 0.0);
        opening.update(PAN_DURATION * 0.5, false);
        assert!(opening.pan_progress() > 0.0 && !opening.is_finished());
        opening.update(PAN_DURATION, false);
        assert!(opening.is_finished());
    }

    #[test]
    fn crawl_ends_on_its_own_and_camera_settles_on_the_game_view() {
        let mut opening = OpeningCrawl::new();
        let game_camera = Camera::new(Vec3::new(0.0, 5.0, 40.0), 16.0 / 9.0);
        assert!(opening.camera(&game_camera).forward().y > 0.5);
        opening.update(crawl_end() + PAN_DURATION + 0.1, false);
        assert!(opening.is_finished());
        let settled = opening.camera(&game_camera).forward();
        assert!((settled - game_camera.forward()).length() < 0.001);
    }
}
