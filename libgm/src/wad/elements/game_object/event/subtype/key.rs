use macros::num_enum;
use std::fmt;

use crate::{
    prelude::*, util::init::num_enum_from, wad::elements::game_object::event::EventSubtype,
};

/// The subtype for [`EventType::Keyboard`], [`EventType::KeyDown`] and [`EventType::KeyUp`].
#[num_enum(u32)]
pub enum Key {
    // TODO: if doesn't match any of the below, then it's probably just chr(value)
    /// Keycode representing no key.
    ///
    /// Mnemonic: `vk_nokey`
    ///
    /// I have no idea why this exists.
    NoKey = 0,

    /// Keycode representing any key.
    ///
    /// Mnemonic: `vk_anykey`
    AnyKey = 1,

    /// Keycode representing the Backspace key.
    ///
    /// Mnemonic: `vk_backspace`
    Backspace = 8,

    /// Keycode representing the Tab key.
    ///
    /// Mnemonic: `vk_tab`
    Tab = 9,

    /// Keycode representing Return / Enter.
    ///
    /// Mnemonic: `vk_enter`
    Enter = 13,

    /// Keycode representing any Shift key.
    ///
    /// Mnemonic: `vk_shift`
    Shift = 16,

    /// Keycode representing any Control key.
    ///
    /// Mnemonic: `vk_control`
    Control = 17,

    /// Keycode representing any Alt key.
    ///
    /// Mnemonic: `vk_alt`
    Alt = 18,

    /// Keycode representing the Pause key.
    ///
    /// Mnemonic: `vk_pause`
    Pause = 19,

    /// Keycode representing the Escape key.
    ///
    /// Mnemonic: `vk_escape`
    Escape = 27,

    /// Keycode representing the Space key.
    ///
    /// Mnemonic: `vk_space`
    Space = 32,

    /// Keycode representing the Page Up key.
    ///
    /// Mnemonic: `vk_pageup`
    PageUp = 33,

    /// Keycode representing the Page Down key.
    ///
    /// Mnemonic: `vk_pagedown`
    PageDown = 34,

    /// Keycode representing the End key.
    ///
    /// Mnemonic: `vk_end`
    End = 35,

    /// Keycode representing the Home key.
    ///
    /// Mnemonic: `vk_home`
    Home = 36,

    /// Keycode representing the Left Arrow key.
    ///
    /// Mnemonic: `vk_left`
    Left = 37,

    /// Keycode representing the Up Arrow key.
    ///
    /// Mnemonic: `vk_up`
    Up = 38,

    /// Keycode representing the Right Arrow key.
    ///
    /// Mnemonic: `vk_right`
    Right = 39,

    /// Keycode representing the Down Arrow key.
    ///
    /// Mnemonic: `vk_down`
    Down = 40,

    /// Keycode representing the Print Screen key.
    ///
    /// Mnemonic: `vk_printscreen`
    PrintScreen = 44,

    /// Keycode representing the Insert key.
    ///
    /// Mnemonic: `vk_insert`
    Insert = 45,

    /// Keycode representing the Delete key.
    ///
    /// Mnemonic: `vk_delete`
    Delete = 46,

    /// Keycode representing the number row 0 key.
    Digit0 = 48,

    /// Keycode representing the number row 1 key.
    Digit1 = 49,

    /// Keycode representing the number row 2 key.
    Digit2 = 50,

    /// Keycode representing the number row 3 key.
    Digit3 = 51,

    /// Keycode representing the number row 4 key.
    Digit4 = 52,

    /// Keycode representing the number row 5 key.
    Digit5 = 53,

    /// Keycode representing the number row 6 key.
    Digit6 = 54,

    /// Keycode representing the number row 7 key.
    Digit7 = 55,

    /// Keycode representing the number row 8 key.
    Digit8 = 56,

    /// Keycode representing the number row 9 key.
    Digit9 = 57,

    /// Keycode representing the A key.
    A = 65,

    /// Keycode representing the B key.
    B = 66,

    /// Keycode representing the C key.
    C = 67,

    /// Keycode representing the D key.
    D = 68,

    /// Keycode representing the E key.
    E = 69,

    /// Keycode representing the F key.
    F = 70,

    /// Keycode representing the G key.
    G = 71,

    /// Keycode representing the H key.
    H = 72,

    /// Keycode representing the I key.
    I = 73,

    /// Keycode representing the J key.
    J = 74,

    /// Keycode representing the K key.
    K = 75,

    /// Keycode representing the L key.
    L = 76,

    /// Keycode representing the M key.
    M = 77,

    /// Keycode representing the N key.
    N = 78,

    /// Keycode representing the O key.
    O = 79,

    /// Keycode representing the P key.
    P = 80,

    /// Keycode representing the Q key.
    Q = 81,

    /// Keycode representing the R key.
    R = 82,

    /// Keycode representing the S key.
    S = 83,

    /// Keycode representing the T key.
    T = 84,

    /// Keycode representing the U key.
    U = 85,

    /// Keycode representing the V key.
    V = 86,

    /// Keycode representing the W key.
    W = 87,

    /// Keycode representing the X key.
    X = 88,

    /// Keycode representing the Y key.
    Y = 89,

    /// Keycode representing the Z key.
    Z = 90,

    /// Keycode representing the 0 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad0`
    Numpad0 = 96,

    /// Keycode representing the 1 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad1`
    Numpad1 = 97,

    /// Keycode representing the 2 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad2`
    Numpad2 = 98,

    /// Keycode representing the 3 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad3`
    Numpad3 = 99,

    /// Keycode representing the 4 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad4`
    Numpad4 = 100,

    /// Keycode representing the 5 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad5`
    Numpad5 = 101,

    /// Keycode representing the 6 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad6`
    Numpad6 = 102,

    /// Keycode representing the 7 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad7`
    Numpad7 = 103,

    /// Keycode representing the 8 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad8`
    Numpad8 = 104,

    /// Keycode representing the 9 key on the numeric keypad.
    ///
    /// Mnemonic: `vk_numpad9`
    Numpad9 = 105,

    /// Keycode representing the Multiply key on the numeric keypad.
    ///
    /// Mnemonic: `vk_multiply`
    Multiply = 106,

    /// Keycode representing the Add key on the numeric keypad.
    ///
    /// Mnemonic: `vk_add`
    Add = 107,

    /// Keycode representing the Subtract key on the numeric keypad.
    ///
    /// Mnemonic: `vk_subtract`
    Subtract = 109,

    /// Keycode representing the Decimal Dot key on the numeric keypad.
    ///
    /// Mnemonic: `vk_decimal`
    Decimal = 110,

    /// Keycode representing the Divide key on the numeric keypad.
    ///
    /// Mnemonic: `vk_divide`
    Divide = 111,

    /// Keycode representing the F1 key.
    ///
    /// Mnemonic: `vk_f1`
    F1 = 112,

    /// Keycode representing the F2 key.
    ///
    /// Mnemonic: `vk_f2`
    F2 = 113,

    /// Keycode representing the F3 key.
    ///
    /// Mnemonic: `vk_f3`
    F3 = 114,

    /// Keycode representing the F4 key.
    ///
    /// Mnemonic: `vk_f4`
    F4 = 115,

    /// Keycode representing the F5 key.
    ///
    /// Mnemonic: `vk_f5`
    F5 = 116,

    /// Keycode representing the F6 key.
    ///
    /// Mnemonic: `vk_f6`
    F6 = 117,

    /// Keycode representing the F7 key.
    ///
    /// Mnemonic: `vk_f7`
    F7 = 118,

    /// Keycode representing the F8 key.
    ///
    /// Mnemonic: `vk_f8`
    F8 = 119,

    /// Keycode representing the F9 key.
    ///
    /// Mnemonic: `vk_f9`
    F9 = 120,

    /// Keycode representing the F10 key.
    ///
    /// Mnemonic: `vk_f10`
    F10 = 121,

    /// Keycode representing the F11 key.
    ///
    /// Mnemonic: `vk_f11`
    F11 = 122,

    /// Keycode representing the F12 key.
    ///
    /// Mnemonic: `vk_f12`
    F12 = 123,

    /// Keycode representing the Left Shift key.
    ///
    /// Mnemonic: `vk_lshift`
    LeftShift = 160,

    /// Keycode representing the Right Shift key.
    ///
    /// Mnemonic: `vk_rshift`
    RightShift = 161,

    /// Keycode representing the Left Control key.
    ///
    /// Mnemonic: `vk_lcontrol`
    LeftControl = 162,

    /// Keycode representing the Right Control key.
    ///
    /// Mnemonic: `vk_rcontrol`
    RightControl = 163,

    /// Keycode representing the Left Alt key.
    ///
    /// Mnemonic: `vk_lalt`
    LeftAlt = 164,

    /// Keycode representing the Right Alt key.
    ///
    /// Mnemonic: `vk_ralt`
    RightAlt = 165,
}

impl Key {
    /// Converts a virtual key constant (`vk_xxxxxx`) into a [`Key`].
    ///
    /// This will only work for "special" keys; not for letters and normal numbers.
    #[must_use]
    pub fn from_virtual_key(string: &str) -> Option<Self> {
        Some(match string {
            "vk_nokey" => Self::NoKey,
            "vk_anykey" => Self::AnyKey,
            "vk_backspace" => Self::Backspace,
            "vk_tab" => Self::Tab,
            "vk_enter" | "vk_return" => Self::Enter,
            "vk_shift" => Self::Shift,
            "vk_control" => Self::Control,
            "vk_alt" => Self::Alt,
            "vk_pause" => Self::Pause,
            "vk_escape" => Self::Escape,
            "vk_space" => Self::Space,
            "vk_pageup" => Self::PageUp,
            "vk_pagedown" => Self::PageDown,
            "vk_end" => Self::End,
            "vk_home" => Self::Home,
            "vk_left" => Self::Left,
            "vk_up" => Self::Up,
            "vk_right" => Self::Right,
            "vk_down" => Self::Down,
            "vk_printscreen" => Self::PrintScreen,
            "vk_insert" => Self::Insert,
            "vk_delete" => Self::Delete,
            "vk_numpad0" => Self::Numpad0,
            "vk_numpad1" => Self::Numpad1,
            "vk_numpad2" => Self::Numpad2,
            "vk_numpad3" => Self::Numpad3,
            "vk_numpad4" => Self::Numpad4,
            "vk_numpad5" => Self::Numpad5,
            "vk_numpad6" => Self::Numpad6,
            "vk_numpad7" => Self::Numpad7,
            "vk_numpad8" => Self::Numpad8,
            "vk_numpad9" => Self::Numpad9,
            "vk_multiply" => Self::Multiply,
            "vk_add" => Self::Add,
            "vk_subtract" => Self::Subtract,
            "vk_decimal" => Self::Decimal,
            "vk_divide" => Self::Divide,
            "vk_f1" => Self::F1,
            "vk_f2" => Self::F2,
            "vk_f3" => Self::F3,
            "vk_f4" => Self::F4,
            "vk_f5" => Self::F5,
            "vk_f6" => Self::F6,
            "vk_f7" => Self::F7,
            "vk_f8" => Self::F8,
            "vk_f9" => Self::F9,
            "vk_f10" => Self::F10,
            "vk_f11" => Self::F11,
            "vk_f12" => Self::F12,
            "vk_lshift" => Self::LeftShift,
            "vk_rshift" => Self::RightShift,
            "vk_lcontrol" => Self::LeftControl,
            "vk_rcontrol" => Self::RightControl,
            "vk_lalt" => Self::LeftAlt,
            "vk_ralt" => Self::RightAlt,
            _ => return None,
        })
    }

    /// Gets the virtual key constant (`vk_xxxxxx`) for this key, if it exists.
    ///
    /// This will succeed for "special" keys but fail for letters and normal numbers.
    #[must_use]
    pub const fn to_virtual_key(self) -> Option<&'static str> {
        Some(match self {
            Self::NoKey => "vk_nokey",
            Self::AnyKey => "vk_anykey",
            Self::Backspace => "vk_backspace",
            Self::Tab => "vk_tab",
            Self::Enter => "vk_enter",
            Self::Shift => "vk_shift",
            Self::Control => "vk_control",
            Self::Alt => "vk_alt",
            Self::Pause => "vk_pause",
            Self::Escape => "vk_escape",
            Self::Space => "vk_space",
            Self::PageUp => "vk_pageup",
            Self::PageDown => "vk_pagedown",
            Self::End => "vk_end",
            Self::Home => "vk_home",
            Self::Left => "vk_left",
            Self::Up => "vk_up",
            Self::Right => "vk_right",
            Self::Down => "vk_down",
            Self::PrintScreen => "vk_printscreen",
            Self::Insert => "vk_insert",
            Self::Delete => "vk_delete",
            Self::Numpad0 => "vk_numpad0",
            Self::Numpad1 => "vk_numpad1",
            Self::Numpad2 => "vk_numpad2",
            Self::Numpad3 => "vk_numpad3",
            Self::Numpad4 => "vk_numpad4",
            Self::Numpad5 => "vk_numpad5",
            Self::Numpad6 => "vk_numpad6",
            Self::Numpad7 => "vk_numpad7",
            Self::Numpad8 => "vk_numpad8",
            Self::Numpad9 => "vk_numpad9",
            Self::Multiply => "vk_multiply",
            Self::Add => "vk_add",
            Self::Subtract => "vk_subtract",
            Self::Decimal => "vk_decimal",
            Self::Divide => "vk_divide",
            Self::F1 => "vk_f1",
            Self::F2 => "vk_f2",
            Self::F3 => "vk_f3",
            Self::F4 => "vk_f4",
            Self::F5 => "vk_f5",
            Self::F6 => "vk_f6",
            Self::F7 => "vk_f7",
            Self::F8 => "vk_f8",
            Self::F9 => "vk_f9",
            Self::F10 => "vk_f10",
            Self::F11 => "vk_f11",
            Self::F12 => "vk_f12",
            Self::LeftShift => "vk_lshift",
            Self::RightShift => "vk_rshift",
            Self::LeftControl => "vk_lcontrol",
            Self::RightControl => "vk_rcontrol",
            Self::LeftAlt => "vk_lalt",
            Self::RightAlt => "vk_ralt",
            _ => return None,
        })
    }

    /// An uppercase, whitespaced identifier for this key.
    #[must_use]
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::NoKey => "No Key",
            Self::AnyKey => "Any Key",
            Self::Backspace => "Backspace",
            Self::Tab => "Tab",
            Self::Enter => "Enter",
            Self::Shift => "Shift",
            Self::Control => "Control",
            Self::Alt => "Alt",
            Self::Pause => "Pause",
            Self::Escape => "Escape",
            Self::Space => "Space",
            Self::PageUp => "Page Up",
            Self::PageDown => "Page Down",
            Self::End => "End",
            Self::Home => "Home",
            Self::Left => "Left",
            Self::Up => "Up",
            Self::Right => "Right",
            Self::Down => "Down",
            Self::PrintScreen => "Print Screen",
            Self::Insert => "Insert",
            Self::Delete => "Delete",
            Self::Digit0 => "0",
            Self::Digit1 => "1",
            Self::Digit2 => "2",
            Self::Digit3 => "3",
            Self::Digit4 => "4",
            Self::Digit5 => "5",
            Self::Digit6 => "6",
            Self::Digit7 => "7",
            Self::Digit8 => "8",
            Self::Digit9 => "9",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Numpad0 => "Numpad 0",
            Self::Numpad1 => "Numpad 1",
            Self::Numpad2 => "Numpad 2",
            Self::Numpad3 => "Numpad 3",
            Self::Numpad4 => "Numpad 4",
            Self::Numpad5 => "Numpad 5",
            Self::Numpad6 => "Numpad 6",
            Self::Numpad7 => "Numpad 7",
            Self::Numpad8 => "Numpad 8",
            Self::Numpad9 => "Numpad 9",
            Self::Multiply => "Multiply",
            Self::Add => "Add",
            Self::Subtract => "Subtract",
            Self::Decimal => "Decimal",
            Self::Divide => "Divide",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::LeftShift => "Left Shift",
            Self::RightShift => "Right Shift",
            Self::LeftControl => "Left Control",
            Self::RightControl => "Right Control",
            Self::LeftAlt => "Left Alt",
            Self::RightAlt => "Right Alt",
        }
    }
}

impl EventSubtype for Key {
    fn parse(subtype: u32) -> Result<Self> {
        num_enum_from(subtype)
    }

    fn build(self) -> u32 {
        self.into()
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}
