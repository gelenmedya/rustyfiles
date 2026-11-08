//! The foreground window's kind and its selected item. Wayland gives apps no access to other
//! windows: always "none".

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    Explorer,
    Desktop,
    Dialog,
    Everything,
    DirectoryOpus,
    None,
}

pub fn classify(_class: &str, _title: &str, _has_shell_view: bool, _has_dui_view: bool) -> TargetKind {
    TargetKind::None
}

pub fn foreground_target() -> (TargetKind, isize) {
    (TargetKind::None, 0)
}

pub fn is_typing(_hwnd: isize) -> bool {
    false
}

pub fn selected_path(_kind: TargetKind, _hwnd: isize) -> Option<PathBuf> {
    None
}
