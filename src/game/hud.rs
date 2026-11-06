use crate::platform::window::TextLabel;
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use crate::renderer::Renderer;

const WIDTH: i32 = FRAMEBUFFER_WIDTH as i32;
const HEIGHT: i32 = FRAMEBUFFER_HEIGHT as i32;
const BAR_HEIGHT: i32 = 13;
const TEXT_SIZE: f32 = 6.0;
const LINE_SPACING: f32 = 9.0;
const PANEL_WIDTH: i32 = 276;
const PANEL_PADDING: f32 = 10.0;
const KEY_COLUMN_WIDTH: f32 = 96.0;
const SPEED_DISPLAY_FACTOR: f32 = 10.0;

pub const WHITE: Color = Color::new(255, 255, 255);
pub const SOFT_WHITE: Color = Color::new(214, 214, 214);
pub const DIM: Color = Color::new(150, 150, 160);
pub const RED: Color = Color::new(255, 64, 64);
pub const NOTICE: Color = Color::new(255, 220, 120);
pub const BLACK: Color = Color::BLACK;

pub const CONTROLS: [(&str, &str); 17] = [
    ("W / S", "VOLAR; ORBITA: CERCA / LEJOS"),
    ("A / D", "GIRAR; ORBITA: RODEAR"),
    ("FLECHAS", "PITCH Y GIRO"),
    ("CLIC DER + MOUSE", "APUNTAR LA NAVE"),
    ("Q / E", "ROLL"),
    ("ESPACIO / CTRL", "SUBIR / BAJAR"),
    ("SHIFT", "TURBO"),
    ("C", "CAMARA SEGUIR / LIBRE"),
    ("RUEDA", "DISTANCIA DE CAMARA"),
    ("O", "ORBITAR / SOLTAR (JUEGO)"),
    ("1 - 5", "ELEGIR PLANETA"),
    ("TAB", "SIGUIENTE DESTINO"),
    ("F", "WARP AL DESTINO"),
    ("- / =", "VELOCIDAD DE ORBITAS"),
    ("M", "METRICAS"),
    ("H", "MOSTRAR AYUDA"),
    ("ESC", "VOLVER AL MENU / SALIR"),
];

#[derive(Debug, Clone, PartialEq)]
pub struct HudInfo {
    pub program_mode: &'static str,
    pub objective: Option<&'static str>,
    pub speed: f32,
    pub flight_mode: String,
    pub camera_mode: &'static str,
    pub destination_number: usize,
    pub destination_name: &'static str,
    pub destination_distance: f32,
    pub warp_status: &'static str,
    pub body_count: usize,
    pub time_scale: f32,
    pub notice: Option<String>,
    pub show_help: bool,
}

pub fn status_line(info: &HudInfo) -> String {
    format!(
        "{} | SPD {:03} | VUELO {} | CAM {} | DEST: {} {} {}U | WARP {}",
        info.program_mode,
        (info.speed * SPEED_DISPLAY_FACTOR).round() as i32,
        info.flight_mode,
        info.camera_mode,
        info.destination_number,
        info.destination_name,
        info.destination_distance.max(0.0).round() as i32,
        info.warp_status,
    )
}

pub fn system_line(info: &HudInfo) -> String {
    format!(
        "SISTEMA - {} CUERPOS | ORBITAS X{:.2}",
        info.body_count, info.time_scale
    )
}

fn draw_frame(renderer: &mut Renderer, x: i32, y: i32, width: i32, height: i32, color: Color) {
    renderer.draw_line_2d(x, y, x + width, y, color);
    renderer.draw_line_2d(x, y + height, x + width, y + height, color);
    renderer.draw_line_2d(x, y, x, y + height, color);
    renderer.draw_line_2d(x + width, y, x + width, y + height, color);
}

pub fn draw_hud(renderer: &mut Renderer, info: &HudInfo) -> Vec<TextLabel> {
    let mut labels = Vec::new();
    let bar_top = HEIGHT - BAR_HEIGHT;
    renderer.draw_crosshair(WHITE);
    renderer.draw_rect_2d(0, bar_top, WIDTH, BAR_HEIGHT, BLACK);
    renderer.draw_line_2d(0, bar_top - 1, WIDTH - 1, bar_top - 1, WHITE);
    labels.push(TextLabel::new(
        status_line(info),
        5.0,
        bar_top as f32 + 3.5,
        TEXT_SIZE,
        SOFT_WHITE,
    ));
    labels.push(
        TextLabel::new(
            system_line(info),
            WIDTH as f32 * 0.5,
            bar_top as f32 - 11.0,
            TEXT_SIZE,
            DIM,
        )
        .centered(),
    );
    if let Some(objective) = info.objective {
        labels.push(
            TextLabel::new(
                objective,
                WIDTH as f32 * 0.5,
                bar_top as f32 - 20.0,
                TEXT_SIZE,
                NOTICE,
            )
            .centered(),
        );
    }
    labels.push(TextLabel::new(
        "H AYUDA",
        WIDTH as f32 - 46.0,
        5.0,
        TEXT_SIZE,
        DIM,
    ));
    if let Some(notice) = &info.notice {
        labels.push(
            TextLabel::new(notice.clone(), WIDTH as f32 * 0.5, 22.0, TEXT_SIZE, NOTICE).centered(),
        );
    }
    if info.show_help {
        draw_help(renderer, &mut labels);
    }
    labels
}

fn draw_help(renderer: &mut Renderer, labels: &mut Vec<TextLabel>) {
    let rows = CONTROLS.len();
    let height = (PANEL_PADDING * 2.0 + LINE_SPACING * (rows as f32 + 2.0)) as i32;
    let left = (WIDTH - PANEL_WIDTH) / 2;
    let top = ((HEIGHT - BAR_HEIGHT - height) / 2).max(2);
    renderer.draw_rect_2d(left, top, PANEL_WIDTH, height, BLACK);
    draw_frame(renderer, left, top, PANEL_WIDTH, height, WHITE);
    labels.push(
        TextLabel::new(
            "== CONTROLES ==",
            WIDTH as f32 * 0.5,
            top as f32 + PANEL_PADDING,
            TEXT_SIZE,
            RED,
        )
        .centered(),
    );
    let first_row = top as f32 + PANEL_PADDING + LINE_SPACING * 2.0;
    for (row, (key, action)) in CONTROLS.iter().enumerate() {
        let y = first_row + row as f32 * LINE_SPACING;
        let x = left as f32 + PANEL_PADDING;
        labels.push(TextLabel::new(*key, x, y, TEXT_SIZE, WHITE));
        labels.push(TextLabel::new(
            *action,
            x + KEY_COLUMN_WIDTH,
            y,
            TEXT_SIZE,
            SOFT_WHITE,
        ));
    }
}
