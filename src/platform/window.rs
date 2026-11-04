use crate::renderer::framebuffer::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use raylib::prelude::*;

pub const WINDOW_WIDTH: i32 = 1280;
pub const WINDOW_HEIGHT: i32 = 720;
const WINDOW_TITLE: &str = "Sistema Solar";

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

    pub fn present(&mut self, pixels: &[u8]) {
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
