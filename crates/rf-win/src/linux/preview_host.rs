//! Windows preview handlers (Office, PDF, fonts… inside the preview pane). Linux has no such
//! system; the in-app viewers cover the common types. Every call is a no-op.

use std::path::Path;

use crate::compat::{GUID, RECT};

pub fn handler_for(_path: &Path) -> Option<GUID> {
    None
}

pub fn is_playable(_path: &Path) -> bool {
    false
}

pub fn media_play(_owner: isize, _path: &Path, _rect: Option<RECT>) {}

pub fn media_pause() {}

pub fn media_seek(_fraction: f64) {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MediaState {
    pub playing: bool,
    /// Seconds.
    pub position: f64,
    pub duration: f64,
}

pub fn media_state() -> Option<MediaState> {
    None
}

pub fn media_reposition(_rect: RECT) {}

pub struct Colors {
    pub background: u32,
    pub text: u32,
}

pub fn show(_owner: isize, _path: &Path, _rect: RECT, _colors: &Colors) -> bool {
    false
}

pub fn hide() {}

pub fn reposition(_rect: RECT) {}
