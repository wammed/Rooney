pub mod buffer;
pub mod pane;

pub use buffer::TextBuffer;
pub use pane::{EditorPane, EditorTab, PaneId, SplitLayout};

use cosmic::iced::keyboard::key::{Code, Physical};

/// Resolves a physical key to its corresponding numeric keypad character if applicable.
/// On Linux/Wayland (e.g. COSMIC desktop), NumLock state is often not tracked or synchronized
/// to client surfaces. This helper maps physical numpad codes to digits and symbols regardless
/// of XKB keysym state.
pub fn resolve_numpad_char(physical_key: &Physical) -> Option<&'static str> {
    if let Physical::Code(code) = physical_key {
        match code {
            Code::Numpad0 => Some("0"),
            Code::Numpad1 => Some("1"),
            Code::Numpad2 => Some("2"),
            Code::Numpad3 => Some("3"),
            Code::Numpad4 => Some("4"),
            Code::Numpad5 => Some("5"),
            Code::Numpad6 => Some("6"),
            Code::Numpad7 => Some("7"),
            Code::Numpad8 => Some("8"),
            Code::Numpad9 => Some("9"),
            Code::NumpadAdd => Some("+"),
            Code::NumpadSubtract => Some("-"),
            Code::NumpadMultiply => Some("*"),
            Code::NumpadDivide => Some("/"),
            Code::NumpadDecimal => Some("."),
            Code::NumpadComma => Some(","),
            Code::NumpadEqual => Some("="),
            _ => None,
        }
    } else {
        None
    }
}
