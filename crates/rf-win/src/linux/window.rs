//! Window helpers GPUI does not expose. Wayland gives clients no control over stacking, global
//! position or focus, so these are no-ops (GPUI sets the app id from the window options).

pub fn set_topmost(_hwnd: isize, _on: bool) -> anyhow::Result<()> {
    Ok(())
}

pub fn is_topmost(_hwnd: isize) -> bool {
    false
}

/// Whether one of our windows is in front. Unknown here: assume yes (callers then skip
/// "bring to front" tricks that Wayland forbids anyway).
pub fn app_in_front() -> bool {
    true
}

/// Window rectangle and maximized state. Not available on Wayland.
pub fn rect(_hwnd: isize) -> Option<([i32; 4], bool)> {
    None
}

pub fn place(_hwnd: isize, _r: [i32; 4]) {}

pub fn maximize(_hwnd: isize) {}

pub fn set_min_size(_hwnd: isize, _size: Option<(u16, u16)>) {}

pub fn set_app_id(_id: &str) {}

pub fn set_dark_mode(_hwnd: isize, _dark: bool) {}

pub fn set_owner(_hwnd: isize, _owner: isize) {}

pub fn hand_foreground(_owner: isize) {}

pub fn bring_to_front(_hwnd: isize) {}

pub fn is_foreground(_hwnd: isize) -> bool {
    false
}
