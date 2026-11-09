use crate::game::upgrades::{describe, UpgradeOption};
use crate::game::weapons::{Inventory, WeaponKind};
use crate::platform::window::TextLabel;
use crate::renderer::framebuffer::{Color, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use crate::renderer::Renderer;

pub const REVEAL_STEP: f32 = 0.35;
const SPIN_RATE: f32 = 18.0;
const TEXT_SIZE: f32 = 6.0;
const LINE: f32 = 9.0;
const CARD_WIDTH: i32 = 140;
const CARD_HEIGHT: i32 = 70;
const CARD_GAP: i32 = 10;
const PANEL_MARGIN: i32 = 10;
const HEADER_HEIGHT: i32 = 26;
const FOOTER_HEIGHT: i32 = 16;

const FRAME: Color = Color::new(150, 150, 160);
const HIGHLIGHT: Color = Color::new(255, 220, 120);
const TITLE: Color = Color::new(170, 92, 255);
const TEXT: Color = Color::new(214, 214, 214);
const SPINNING: Color = Color::new(120, 120, 130);
const BACKDROP: Color = Color::new(8, 4, 16);

pub struct Roulette {
    options: Vec<UpgradeOption>,
    elapsed: f32,
    selected: usize,
    level: u32,
}

fn frame(renderer: &mut Renderer, x: i32, y: i32, width: i32, height: i32, color: Color) {
    renderer.draw_line_2d(x, y, x + width, y, color);
    renderer.draw_line_2d(x, y + height, x + width, y + height, color);
    renderer.draw_line_2d(x, y, x, y + height, color);
    renderer.draw_line_2d(x + width, y, x + width, y + height, color);
}

impl Roulette {
    pub fn new(options: Vec<UpgradeOption>, level: u32) -> Self {
        Self {
            options,
            elapsed: 0.0,
            selected: 0,
            level,
        }
    }

    pub fn options(&self) -> &[UpgradeOption] {
        &self.options
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        let before = self.revealed_count();
        self.elapsed += delta_time;
        self.revealed_count() != before
    }

    fn revealed_count(&self) -> usize {
        ((self.elapsed / REVEAL_STEP) as usize).min(self.options.len())
    }

    pub fn is_ready(&self) -> bool {
        self.revealed_count() == self.options.len()
    }

    pub fn move_selection(&mut self, step: i32) {
        if self.options.is_empty() {
            return;
        }
        let count = self.options.len() as i32;
        self.selected = (self.selected as i32 + step).rem_euclid(count) as usize;
    }

    pub fn choose(&mut self, index: usize) -> Option<UpgradeOption> {
        if index >= self.options.len() {
            return None;
        }
        self.selected = index;
        self.confirm()
    }

    pub fn confirm(&self) -> Option<UpgradeOption> {
        self.is_ready()
            .then(|| self.options.get(self.selected).copied())
            .flatten()
    }

    pub fn draw(&self, renderer: &mut Renderer, inventory: &Inventory) -> Vec<TextLabel> {
        let mut labels = Vec::new();
        let count = self.options.len().max(1) as i32;
        let cards_width = count * CARD_WIDTH + (count - 1) * CARD_GAP;
        let panel_width = cards_width + PANEL_MARGIN * 2;
        let panel_height = HEADER_HEIGHT + CARD_HEIGHT + FOOTER_HEIGHT + PANEL_MARGIN;
        let left = (FRAMEBUFFER_WIDTH as i32 - panel_width) / 2;
        let top = (FRAMEBUFFER_HEIGHT as i32 - panel_height) / 2 - 10;
        let center_x = FRAMEBUFFER_WIDTH as f32 * 0.5;
        renderer.draw_rect_2d(left, top, panel_width, panel_height, BACKDROP);
        frame(renderer, left, top, panel_width, panel_height, FRAME);
        labels.push(
            TextLabel::new(
                format!("== NIVEL {} - ELIGE UNA MEJORA ==", self.level),
                center_x,
                top as f32 + 8.0,
                TEXT_SIZE,
                TITLE,
            )
            .centered(),
        );

        let revealed = self.revealed_count();
        let card_top = top + HEADER_HEIGHT;
        for (index, option) in self.options.iter().enumerate() {
            let card_left = left + PANEL_MARGIN + index as i32 * (CARD_WIDTH + CARD_GAP);
            let card_center = card_left as f32 + CARD_WIDTH as f32 * 0.5;
            let is_revealed = index < revealed;
            let is_selected = is_revealed && self.is_ready() && index == self.selected;
            let color = if is_selected { HIGHLIGHT } else { FRAME };
            frame(renderer, card_left, card_top, CARD_WIDTH, CARD_HEIGHT, color);
            if is_selected {
                frame(renderer, card_left + 2, card_top + 2, CARD_WIDTH - 4, CARD_HEIGHT - 4, color);
            }
            labels.push(
                TextLabel::new(
                    format!("{}", index + 1),
                    card_left as f32 + 5.0,
                    card_top as f32 + 4.0,
                    TEXT_SIZE,
                    color,
                )
                .centered(),
            );
            if !is_revealed {
                let spin = (self.elapsed * SPIN_RATE) as usize + index * 2;
                let name = WeaponKind::ALL[spin % WeaponKind::ALL.len()].name();
                labels.push(
                    TextLabel::new(
                        format!("? {} ?", name),
                        card_center,
                        card_top as f32 + CARD_HEIGHT as f32 * 0.45,
                        TEXT_SIZE,
                        SPINNING,
                    )
                    .centered(),
                );
                continue;
            }
            let (title, lines) = describe(*option, inventory);
            labels.push(
                TextLabel::new(title, card_center, card_top as f32 + 12.0, TEXT_SIZE, HIGHLIGHT)
                    .centered(),
            );
            for (row, line) in lines.into_iter().enumerate() {
                labels.push(
                    TextLabel::new(
                        line,
                        card_center,
                        card_top as f32 + 28.0 + row as f32 * LINE,
                        TEXT_SIZE,
                        TEXT,
                    )
                    .centered(),
                );
            }
        }

        let footer = if self.is_ready() {
            "A / D ELEGIR   ENTER CONFIRMAR   1 - 3 ELECCION RAPIDA"
        } else {
            "GIRANDO..."
        };
        labels.push(
            TextLabel::new(
                footer,
                center_x,
                (card_top + CARD_HEIGHT + 5) as f32,
                TEXT_SIZE,
                FRAME,
            )
            .centered(),
        );
        labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roulette() -> Roulette {
        Roulette::new(
            vec![
                UpgradeOption::Improve(WeaponKind::Normal),
                UpgradeOption::Unlock(WeaponKind::Laser),
                UpgradeOption::Repair,
            ],
            2,
        )
    }

    #[test]
    fn cards_reveal_one_by_one_before_choosing() {
        let mut wheel = roulette();
        assert!(wheel.confirm().is_none());
        assert!(wheel.update(REVEAL_STEP + 0.01));
        assert!(!wheel.is_ready());
        wheel.update(REVEAL_STEP * 3.0);
        assert!(wheel.is_ready());
        assert_eq!(wheel.confirm(), Some(UpgradeOption::Improve(WeaponKind::Normal)));
    }

    #[test]
    fn selection_wraps_and_quick_choice_works() {
        let mut wheel = roulette();
        wheel.update(5.0);
        wheel.move_selection(-1);
        assert_eq!(wheel.selected(), 2);
        wheel.move_selection(1);
        assert_eq!(wheel.selected(), 0);
        assert_eq!(wheel.choose(1), Some(UpgradeOption::Unlock(WeaponKind::Laser)));
        assert_eq!(wheel.choose(7), None);
    }
}
