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
    G,
    H,
    O,
    Tab,
    Space,
    LeftControl,
    LeftShift,
    Up,
    Down,
    Left,
    Right,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    M,
    Minus,
    Equal,
    Escape,
    Enter,
}

impl Key {
    pub const ALL: [Key; 29] = [
        Key::W,
        Key::A,
        Key::S,
        Key::D,
        Key::Q,
        Key::E,
        Key::C,
        Key::F,
        Key::G,
        Key::H,
        Key::O,
        Key::Tab,
        Key::Space,
        Key::LeftControl,
        Key::LeftShift,
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
        Key::M,
        Key::Minus,
        Key::Equal,
        Key::Escape,
        Key::Enter,
    ];

    pub const DIGITS: [Key; 5] = [
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
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
            Key::G => KeyboardKey::KEY_G,
            Key::H => KeyboardKey::KEY_H,
            Key::O => KeyboardKey::KEY_O,
            Key::Tab => KeyboardKey::KEY_TAB,
            Key::Space => KeyboardKey::KEY_SPACE,
            Key::LeftControl => KeyboardKey::KEY_LEFT_CONTROL,
            Key::LeftShift => KeyboardKey::KEY_LEFT_SHIFT,
            Key::Up => KeyboardKey::KEY_UP,
            Key::Down => KeyboardKey::KEY_DOWN,
            Key::Left => KeyboardKey::KEY_LEFT,
            Key::Right => KeyboardKey::KEY_RIGHT,
            Key::Digit1 => KeyboardKey::KEY_ONE,
            Key::Digit2 => KeyboardKey::KEY_TWO,
            Key::Digit3 => KeyboardKey::KEY_THREE,
            Key::Digit4 => KeyboardKey::KEY_FOUR,
            Key::Digit5 => KeyboardKey::KEY_FIVE,
            Key::M => KeyboardKey::KEY_M,
            Key::Minus => KeyboardKey::KEY_MINUS,
            Key::Equal => KeyboardKey::KEY_EQUAL,
            Key::Escape => KeyboardKey::KEY_ESCAPE,
            Key::Enter => KeyboardKey::KEY_ENTER,
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

#[derive(Debug, Clone, Default)]
pub struct InputState {
    held_keys: [bool; KEY_COUNT],
    pressed_keys: [bool; KEY_COUNT],
    held_buttons: [bool; MOUSE_BUTTON_COUNT],
    pressed_buttons: [bool; MOUSE_BUTTON_COUNT],
    mouse_delta: Vec2,
    wheel: f32,
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
