use crate::renderer::framebuffer::{Color, Framebuffer};

const GLYPH_WIDTH: i32 = 3;
const GLYPH_HEIGHT: usize = 5;
const CHARACTER_ADVANCE: i32 = 4;
const LINE_ADVANCE: i32 = 7;
const MARGIN: i32 = 4;
const FPS_SMOOTHING: f32 = 0.05;

fn glyph(character: char) -> [u8; GLYPH_HEIGHT] {
    match character {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'P' => [0b111, 0b101, 0b111, 0b100, 0b100],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b111, 0b100, 0b111, 0b001, 0b111],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'W' => [0b101, 0b101, 0b101, 0b111, 0b101],
        _ => [0; GLYPH_HEIGHT],
    }
}

fn draw_text(framebuffer: &mut Framebuffer, x: i32, y: i32, text: &str, color: Color) {
    for (index, character) in text.chars().enumerate() {
        let left = x + index as i32 * CHARACTER_ADVANCE;
        for (row, bits) in glyph(character).iter().enumerate() {
            for column in 0..GLYPH_WIDTH {
                if bits & (1 << (GLYPH_WIDTH - 1 - column)) != 0 {
                    framebuffer.set_pixel(left + column, y + row as i32, color);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Metrics {
    pub fps: f32,
    pub triangles_processed: usize,
    pub triangles_drawn: usize,
    pub objects_culled: usize,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, delta_time: f32) {
        if delta_time > 0.0 {
            self.fps += (1.0 / delta_time - self.fps) * FPS_SMOOTHING;
        }
    }

    pub fn record_frame(&mut self, processed: usize, drawn: usize, culled: usize) {
        self.triangles_processed = processed;
        self.triangles_drawn = drawn;
        self.objects_culled = culled;
    }

    pub fn draw(&self, framebuffer: &mut Framebuffer, color: Color) {
        let lines = [
            format!("FPS {}", self.fps as u32),
            format!("TRI {}", self.triangles_processed),
            format!("DRW {}", self.triangles_drawn),
            format!("CUL {}", self.objects_culled),
        ];
        for (index, line) in lines.iter().enumerate() {
            let y = MARGIN + index as i32 * LINE_ADVANCE;
            draw_text(framebuffer, MARGIN, y, line, color);
        }
    }
}
