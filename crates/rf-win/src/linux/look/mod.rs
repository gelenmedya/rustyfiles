//! DirectLook's system-wide parts. The key filter and the image decoder are shared with Windows;
//! the global keyboard hook and "what is selected in the foreground window" can't exist on
//! Wayland, so on Linux DirectLook is opened from rustyfiles itself (Space) or its command line.

#[path = "../../look/keys.rs"]
pub mod keys;
#[path = "../../look/decode.rs"]
pub mod decode;
pub mod hook;
pub mod pdf;
pub mod target;
pub mod win;
