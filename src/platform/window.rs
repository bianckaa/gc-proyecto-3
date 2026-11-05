use crate::renderer::framebuffer::{Color as PixelColor, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use raylib::prelude::*;

pub const WINDOW_WIDTH: i32 = 1280;
pub const WINDOW_HEIGHT: i32 = 720;
const WINDOW_TITLE: &str = "Sistema Solar";
const TARGET_FPS: u32 = 120;
pub const GLYPH_ADVANCE_RATIO: f32 = 0.75;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLabel {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: PixelColor,
    pub align: TextAlign,
}

impl TextLabel {
    pub fn new(text: impl Into<String>, x: f32, y: f32, size: f32, color: PixelColor) -> Self {
        Self {
            text: text.into(),
            x,
            y,
            size,
            color,
            align: TextAlign::Left,
        }
    }

    pub fn centered(mut self) -> Self {
        self.align = TextAlign::Center;
        self
    }

    pub fn width(&self) -> f32 {
        self.text.chars().count() as f32 * self.size * GLYPH_ADVANCE_RATIO
    }
}

pub struct Window {
    handle: RaylibHandle,
    thread: RaylibThread,
    texture: Texture2D,
}

impl Window {
    pub fn open() -> Self {
        let (mut handle, thread) = raylib::init()
            .size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .title(WINDOW_TITLE)
            .resizable()
            .build();
        handle.set_window_min_size(FRAMEBUFFER_WIDTH as i32, FRAMEBUFFER_HEIGHT as i32);
        handle.set_target_fps(TARGET_FPS);
        handle.set_exit_key(None);

        let image = Image::gen_image_color(
            FRAMEBUFFER_WIDTH as i32,
            FRAMEBUFFER_HEIGHT as i32,
            Color::BLACK,
        );
        let texture = handle
            .load_texture_from_image(&thread, &image)
            .expect("no se pudo crear la textura del framebuffer");
        texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

        Self {
            handle,
            thread,
            texture,
        }
    }

    pub fn should_close(&self) -> bool {
        self.handle.window_should_close()
    }

    pub fn handle(&self) -> &RaylibHandle {
        &self.handle
    }

    pub fn present(&mut self, pixels: &[u8], labels: &[TextLabel]) {
        self.texture
            .update_texture(pixels)
            .expect("no se pudo actualizar la textura del framebuffer");

        let destination = self.fitted_destination();
        let source = Rectangle::new(
            0.0,
            0.0,
            FRAMEBUFFER_WIDTH as f32,
            FRAMEBUFFER_HEIGHT as f32,
        );
        let mut drawing = self.handle.begin_drawing(&self.thread);
        drawing.clear_background(Color::BLACK);
        drawing.draw_texture_pro(
            &self.texture,
            source,
            destination,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        let scale = destination.width / FRAMEBUFFER_WIDTH as f32;
        for label in labels {
            let size = (label.size * scale).round().max(1.0);
            let advance = label.size * GLYPH_ADVANCE_RATIO * scale;
            let left = match label.align {
                TextAlign::Left => label.x,
                TextAlign::Center => label.x - label.width() * 0.5,
            };
            let start_x = destination.x + left * scale;
            let y = (destination.y + label.y * scale).round() as i32;
            let color = Color::new(label.color.r, label.color.g, label.color.b, 255);
            let mut buffer = [0u8; 4];
            for (index, character) in label.text.chars().enumerate() {
                if character == ' ' {
                    continue;
                }
                let x = (start_x + index as f32 * advance).round() as i32;
                drawing.draw_text(character.encode_utf8(&mut buffer), x, y, size as i32, color);
            }
        }
    }

    fn fitted_destination(&self) -> Rectangle {
        let screen_width = self.handle.get_screen_width() as f32;
        let screen_height = self.handle.get_screen_height() as f32;
        let scale = (screen_width / FRAMEBUFFER_WIDTH as f32)
            .min(screen_height / FRAMEBUFFER_HEIGHT as f32);
        let width = (FRAMEBUFFER_WIDTH as f32 * scale).floor();
        let height = (FRAMEBUFFER_HEIGHT as f32 * scale).floor();
        Rectangle::new(
            ((screen_width - width) * 0.5).floor(),
            ((screen_height - height) * 0.5).floor(),
            width,
            height,
        )
    }
}
