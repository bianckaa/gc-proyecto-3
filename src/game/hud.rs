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
pub const HEALTH: Color = Color::new(72, 220, 96);
pub const EXPERIENCE: Color = Color::new(170, 92, 255);
pub const BLACK: Color = Color::BLACK;

const PROGRESS_BAR_WIDTH: i32 = 108;
const PROGRESS_BAR_HEIGHT: i32 = 4;
const LOADOUT_LINE: f32 = 8.0;

pub const CONTROLS: [(&str, &str); 20] = [
    ("W / S", "VOLAR; ORBITA: CERCA / LEJOS"),
    ("A / D", "GIRAR; ORBITA: RODEAR"),
    ("FLECHAS", "PITCH Y GIRO"),
    ("CLIC DER + MOUSE", "APUNTAR LA NAVE"),
    ("Q / E", "ROLL"),
    ("ESPACIO / CTRL", "SUBIR / BAJAR"),
    ("SHIFT", "TURBO"),
    ("R", "LOOP EVASIVO (JUEGO)"),
    ("C", "CAMARA SEGUIR / LIBRE"),
    ("RUEDA", "DISTANCIA DE CAMARA"),
    ("1 - 5", "ELEGIR PLANETA (MENU)"),
    ("TAB", "SIGUIENTE DESTINO (MENU)"),
    ("F", "WARP AL DESTINO (MENU)"),
    ("- / =", "VELOCIDAD DE ORBITAS"),
    ("I", "EDITAR SEMILLA (MENU)"),
    ("R", "SEMILLA ALEATORIA (MENU)"),
    ("M", "METRICAS"),
    ("G", "GRAVEDAD"),
    ("H", "MOSTRAR AYUDA"),
    ("ESC", "VOLVER AL MENU / SALIR"),
];

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressHudInfo {
    pub health: f32,
    pub max_health: f32,
    pub health_ratio: f32,
    pub level: u32,
    pub experience: u32,
    pub experience_required: u32,
    pub experience_ratio: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HudInfo {
    pub program_mode: &'static str,
    pub objective: Option<&'static str>,
    pub run_time_remaining: Option<f32>,
    pub run_seed: u32,
    pub seed_input: Option<String>,
    pub progression: Option<ProgressHudInfo>,
    pub speed: f32,
    pub flight_mode: String,
    pub camera_mode: &'static str,
    pub destination_number: usize,
    pub destination_name: &'static str,
    pub destination_distance: f32,
    pub warp_status: &'static str,
    pub notice: Option<String>,
    pub show_help: bool,
    pub weapons: Vec<String>,
    pub buffs: Vec<String>,
    pub target: Option<(String, f32)>,
}

pub fn status_line(info: &HudInfo) -> String {
    if info.program_mode == "JUEGO" {
        return format!(
            "{} | SPD {:03} | VUELO {} | CAM {}",
            info.program_mode,
            (info.speed * SPEED_DISPLAY_FACTOR).round() as i32,
            info.flight_mode,
            info.camera_mode,
        );
    }
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

pub fn format_time(seconds: f32) -> String {
    let total = seconds.max(0.0).ceil() as u32;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn draw_frame(renderer: &mut Renderer, x: i32, y: i32, width: i32, height: i32, color: Color) {
    renderer.draw_line_2d(x, y, x + width, y, color);
    renderer.draw_line_2d(x, y + height, x + width, y + height, color);
    renderer.draw_line_2d(x, y, x, y + height, color);
    renderer.draw_line_2d(x + width, y, x + width, y + height, color);
}

pub fn draw_hud(
    renderer: &mut Renderer,
    info: &HudInfo,
    crosshair_position: Option<(i32, i32)>,
) -> Vec<TextLabel> {
    let mut labels = Vec::new();
    let bar_top = HEIGHT - BAR_HEIGHT;
    if let Some((x, y)) = crosshair_position {
        renderer.draw_crosshair_at(x, y, WHITE);
    } else {
        renderer.draw_crosshair(WHITE);
    }
    renderer.draw_rect_2d(0, bar_top, WIDTH, BAR_HEIGHT, BLACK);
    renderer.draw_line_2d(0, bar_top - 1, WIDTH - 1, bar_top - 1, WHITE);
    if let Some(progress) = &info.progression {
        draw_progress(renderer, &mut labels, progress, bar_top);
    }
    labels.push(TextLabel::new(
        status_line(info),
        5.0,
        bar_top as f32 + 3.5,
        TEXT_SIZE,
        SOFT_WHITE,
    ));
    if let Some(remaining) = info.run_time_remaining {
        labels.push(
            TextLabel::new(
                format!("TIEMPO {}", format_time(remaining)),
                WIDTH as f32 * 0.5,
                5.0,
                TEXT_SIZE,
                WHITE,
            )
            .centered(),
        );
    }
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
    let seed_text = match &info.seed_input {
        Some(input) => format!("SEMILLA > {}_ | ENTER CONFIRMAR | ESC CANCELAR", input),
        None if info.program_mode == "MENU" => {
            format!("SEMILLA {} | I EDITAR | R ALEATORIA", info.run_seed)
        }
        None => format!("SEED {}", info.run_seed),
    };
    labels.push(TextLabel::new(seed_text, 5.0, 5.0, TEXT_SIZE, DIM));
    if let Some(notice) = &info.notice {
        labels.push(
            TextLabel::new(notice.clone(), WIDTH as f32 * 0.5, 22.0, TEXT_SIZE, NOTICE).centered(),
        );
    }
    draw_loadout(&mut labels, info, bar_top);
    if let Some((name, ratio)) = &info.target {
        draw_target(renderer, &mut labels, name, *ratio);
    }
    if info.show_help {
        draw_help(renderer, &mut labels);
    }
    labels
}

fn draw_target(renderer: &mut Renderer, labels: &mut Vec<TextLabel>, name: &str, ratio: f32) {
    let x = (WIDTH - PROGRESS_BAR_WIDTH) / 2;
    let y = 24;
    labels.push(
        TextLabel::new(name, WIDTH as f32 * 0.5, y as f32 - 8.0, TEXT_SIZE, RED).centered(),
    );
    draw_progress_bar(renderer, x, y, ratio, RED);
}

fn draw_loadout(labels: &mut Vec<TextLabel>, info: &HudInfo, bar_top: i32) {
    let right = WIDTH as f32 - 5.0;
    let mut y = bar_top as f32 - LOADOUT_LINE * (info.weapons.len() + info.buffs.len()) as f32 - 6.0;
    for line in info.weapons.iter() {
        let label = TextLabel::new(line.clone(), 0.0, y, TEXT_SIZE, SOFT_WHITE);
        let x = right - label.width();
        labels.push(TextLabel::new(line.clone(), x, y, TEXT_SIZE, SOFT_WHITE));
        y += LOADOUT_LINE;
    }
    for line in info.buffs.iter() {
        let label = TextLabel::new(line.clone(), 0.0, y, TEXT_SIZE, NOTICE);
        let x = right - label.width();
        labels.push(TextLabel::new(line.clone(), x, y, TEXT_SIZE, NOTICE));
        y += LOADOUT_LINE;
    }
}

fn draw_progress(
    renderer: &mut Renderer,
    labels: &mut Vec<TextLabel>,
    progress: &ProgressHudInfo,
    bar_top: i32,
) {
    let health_y = bar_top - 13;
    let experience_y = bar_top - 7;
    draw_progress_bar(renderer, 5, health_y, progress.health_ratio, HEALTH);
    draw_progress_bar(
        renderer,
        5,
        experience_y,
        progress.experience_ratio,
        EXPERIENCE,
    );
    labels.push(TextLabel::new(
        format!("HP {:.0}/{:.0}", progress.health, progress.max_health),
        118.0,
        health_y as f32 - 1.0,
        TEXT_SIZE,
        HEALTH,
    ));
    labels.push(TextLabel::new(
        format!(
            "NV {}  XP {}/{}",
            progress.level, progress.experience, progress.experience_required
        ),
        118.0,
        experience_y as f32 - 1.0,
        TEXT_SIZE,
        EXPERIENCE,
    ));
}

fn draw_progress_bar(renderer: &mut Renderer, x: i32, y: i32, ratio: f32, color: Color) {
    renderer.draw_rect_2d(x, y, PROGRESS_BAR_WIDTH, PROGRESS_BAR_HEIGHT, BLACK);
    let fill = ((PROGRESS_BAR_WIDTH - 2) as f32 * ratio.clamp(0.0, 1.0)).round() as i32;
    if fill > 0 {
        renderer.draw_rect_2d(x + 1, y + 1, fill, PROGRESS_BAR_HEIGHT - 2, color);
    }
    draw_frame(renderer, x, y, PROGRESS_BAR_WIDTH, PROGRESS_BAR_HEIGHT, DIM);
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

#[cfg(test)]
mod tests {
    use super::format_time;

    #[test]
    fn run_time_is_displayed_as_minutes_and_seconds() {
        assert_eq!(format_time(900.0), "15:00");
        assert_eq!(format_time(61.2), "01:02");
        assert_eq!(format_time(0.0), "00:00");
    }
}
