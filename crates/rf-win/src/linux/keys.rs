//! Keyboard-layout helpers. On Windows, Shift+digit needs translating through the layout; GPUI's
//! Linux backend (xkbcommon) already reports the shifted character. TODO(linux): verify.

pub fn shifted_char(_vk: u16) -> Option<String> {
    None
}
