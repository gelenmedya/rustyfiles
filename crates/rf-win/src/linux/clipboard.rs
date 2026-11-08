//! File clipboard. TODO(linux, phase 3): `text/uri-list` + `x-special/gnome-copied-files`
//! (cut vs copy), compatible with Nautilus / Dolphin / Thunar. Until then copy/paste of files
//! stays inside the app (the app keeps its own list when the system clipboard fails).

use std::path::PathBuf;

use super::fileops::Transfer;
use crate::compat::HWND;

pub fn clear(_owner: HWND) -> anyhow::Result<()> {
    Ok(())
}

/// Changes whenever the clipboard content changes (0 = unknown).
pub fn sequence() -> u32 {
    0
}

pub fn set_files(_owner: HWND, _paths: &[PathBuf], _kind: Transfer) -> anyhow::Result<()> {
    Err(super::unsupported("file clipboard"))
}

pub fn has_files() -> bool {
    false
}

pub fn get_files(_owner: HWND) -> anyhow::Result<Option<(Vec<PathBuf>, Transfer)>> {
    Ok(None)
}
