use crate::engine::math::Vec2;
use raylib::prelude::{KeyboardKey, MouseButton as RaylibMouseButton, RaylibHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    W,
    A,
    S,
    D,
    Q,
    E,
    C,
    F,
    H,
    I,
    R,
    O,
    Tab,
    Space,
    LeftControl,
    LeftShift,
    Up,
    Down,
    Left,
    Right,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    M,
    Minus,
    Equal,
    Backspace,
    Enter,
    Escape,
}

impl Key {
    pub const ALL: [Key; 36] = [
        Key::W,
        Key::A,
        Key::S,
        Key::D,
        Key::Q,
        Key::E,
        Key::C,
        Key::F,
        Key::H,
        Key::I,
        Key::R,
        Key::O,
        Key::Tab,
        Key::Space,
        Key::LeftControl,
        Key::LeftShift,
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Digit0,
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
        Key::Digit6,
        Key::Digit7,
        Key::Digit8,
        Key::Digit9,
        Key::M,
        Key::Minus,
        Key::Equal,
        Key::Backspace,
        Key::Enter,
        Key::Escape,
    ];

    pub const DIGITS: [Key; 5] = [
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
    ];

    pub const NUMERIC_DIGITS: [Key; 10] = [
        Key::Digit0,
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
        Key::Digit6,
        Key::Digit7,
        Key::Digit8,
        Key::Digit9,
    ];

    fn raylib_key(self) -> KeyboardKey {
        match self {
            Key::W => KeyboardKey::KEY_W,
            Key::A => KeyboardKey::KEY_A,
            Key::S => KeyboardKey::KEY_S,
            Key::D => KeyboardKey::KEY_D,
            Key::Q => KeyboardKey::KEY_Q,
            Key::E => KeyboardKey::KEY_E,
            Key::C => KeyboardKey::KEY_C,
            Key::F => KeyboardKey::KEY_F,
            Key::H => KeyboardKey::KEY_H,
            Key::I => KeyboardKey::KEY_I,
            Key::R => KeyboardKey::KEY_R,
            Key::O => KeyboardKey::KEY_O,
            Key::Tab => KeyboardKey::KEY_TAB,
            Key::Space => KeyboardKey::KEY_SPACE,
            Key::LeftControl => KeyboardKey::KEY_LEFT_CONTROL,
            Key::LeftShift => KeyboardKey::KEY_LEFT_SHIFT,
            Key::Up => KeyboardKey::KEY_UP,
            Key::Down => KeyboardKey::KEY_DOWN,
            Key::Left => KeyboardKey::KEY_LEFT,
            Key::Right => KeyboardKey::KEY_RIGHT,
            Key::Digit0 => KeyboardKey::KEY_ZERO,
            Key::Digit1 => KeyboardKey::KEY_ONE,
            Key::Digit2 => KeyboardKey::KEY_TWO,
            Key::Digit3 => KeyboardKey::KEY_THREE,
            Key::Digit4 => KeyboardKey::KEY_FOUR,
            Key::Digit5 => KeyboardKey::KEY_FIVE,
            Key::Digit6 => KeyboardKey::KEY_SIX,
            Key::Digit7 => KeyboardKey::KEY_SEVEN,
            Key::Digit8 => KeyboardKey::KEY_EIGHT,
            Key::Digit9 => KeyboardKey::KEY_NINE,
            Key::M => KeyboardKey::KEY_M,
            Key::Minus => KeyboardKey::KEY_MINUS,
            Key::Equal => KeyboardKey::KEY_EQUAL,
            Key::Backspace => KeyboardKey::KEY_BACKSPACE,
            Key::Enter => KeyboardKey::KEY_ENTER,
            Key::Escape => KeyboardKey::KEY_ESCAPE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
}

impl MouseButton {
    pub const ALL: [MouseButton; 2] = [MouseButton::Left, MouseButton::Right];

    fn raylib_button(self) -> RaylibMouseButton {
        match self {
            MouseButton::Left => RaylibMouseButton::MOUSE_BUTTON_LEFT,
            MouseButton::Right => RaylibMouseButton::MOUSE_BUTTON_RIGHT,
        }
    }
}

const KEY_COUNT: usize = Key::ALL.len();
const MOUSE_BUTTON_COUNT: usize = MouseButton::ALL.len();

#[derive(Debug, Clone)]
pub struct InputState {
    held_keys: [bool; KEY_COUNT],
    pressed_keys: [bool; KEY_COUNT],
    held_buttons: [bool; MOUSE_BUTTON_COUNT],
    pressed_buttons: [bool; MOUSE_BUTTON_COUNT],
    mouse_delta: Vec2,
    wheel: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            held_keys: [false; KEY_COUNT],
            pressed_keys: [false; KEY_COUNT],
            held_buttons: [false; MOUSE_BUTTON_COUNT],
            pressed_buttons: [false; MOUSE_BUTTON_COUNT],
            mouse_delta: Vec2::default(),
            wheel: 0.0,
        }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture(&mut self, handle: &RaylibHandle) {
        for key in Key::ALL {
            let index = key as usize;
            self.held_keys[index] = handle.is_key_down(key.raylib_key());
            self.pressed_keys[index] |= handle.is_key_pressed(key.raylib_key());
        }
        for button in MouseButton::ALL {
            let index = button as usize;
            self.held_buttons[index] = handle.is_mouse_button_down(button.raylib_button());
            self.pressed_buttons[index] |= handle.is_mouse_button_pressed(button.raylib_button());
        }
        let delta = handle.get_mouse_delta();
        self.mouse_delta = self.mouse_delta + Vec2::new(delta.x, delta.y);
        self.wheel += handle.get_mouse_wheel_move();
    }

    pub fn consume_events(&mut self) {
        self.pressed_keys = [false; KEY_COUNT];
        self.pressed_buttons = [false; MOUSE_BUTTON_COUNT];
        self.mouse_delta = Vec2::default();
        self.wheel = 0.0;
    }

    pub fn is_held(&self, key: Key) -> bool {
        self.held_keys[key as usize]
    }

    pub fn was_pressed(&self, key: Key) -> bool {
        self.pressed_keys[key as usize]
    }

    pub fn axis(&self, positive: Key, negative: Key) -> f32 {
        self.is_held(positive) as i32 as f32 - self.is_held(negative) as i32 as f32
    }

    pub fn pressed_digit(&self) -> Option<usize> {
        Key::DIGITS.iter().position(|&key| self.was_pressed(key))
    }

    pub fn pressed_numeric_digit(&self) -> Option<u8> {
        Key::NUMERIC_DIGITS
            .iter()
            .position(|&key| self.was_pressed(key))
            .map(|digit| digit as u8)
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.held_buttons[button as usize]
    }

    pub fn was_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons[button as usize]
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    pub fn wheel(&self) -> f32 {
        self.wheel
    }
}
