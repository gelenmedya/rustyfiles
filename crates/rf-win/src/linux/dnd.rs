//! Dragging files out to other apps. TODO(linux, phase 3): needs drag sources in GPUI's Linux
//! backend (Wayland `wl_data_source`, X11 XDND) offering `text/uri-list`.

use std::path::{Path, PathBuf};

use super::fileops::Transfer;
use crate::compat::{DROPEFFECT, HWND};

pub fn drag_out(_owner: HWND, _ui_thread: u32, _items: &[PathBuf]) -> anyhow::Result<DROPEFFECT> {
    Err(super::unsupported("dragging files to other apps"))
}

/// What a plain drop does, like file managers do: move within one filesystem, copy across
/// filesystems; Ctrl forces copy, Shift forces move.
pub fn default_transfer(sources: &[PathBuf], dest_dir: &Path, ctrl: bool, shift: bool) -> Transfer {
    if ctrl {
        return Transfer::Copy;
    }
    if shift {
        return Transfer::Move;
    }
    use std::os::unix::fs::MetadataExt;
    let dev = |p: &Path| std::fs::metadata(p).map(|m| m.dev()).ok();
    let dest = dev(dest_dir);
    if dest.is_some() && sources.iter().all(|s| dev(s) == dest) { Transfer::Move } else { Transfer::Copy }
}

/// Dropping a folder into itself (or its own subfolder), or items onto the folder they are in.
pub fn is_invalid_drop(sources: &[PathBuf], dest_dir: &Path) -> bool {
    sources.iter().any(|s| dest_dir.starts_with(s)) || sources.iter().all(|s| s.parent() == Some(dest_dir))
}
