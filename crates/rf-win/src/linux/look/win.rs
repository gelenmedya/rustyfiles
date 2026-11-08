//! Window helpers for the DirectLook window. Wayland doesn't let a client position itself or
//! stay on top, so most are no-ops; the work area is a sensible default.

/// [x, y, width, height] of the work area under the mouse. Unknown on Wayland.
pub fn work_area_at_cursor() -> [f32; 4] {
    [0.0, 0.0, 1920.0, 1080.0]
}

pub fn work_area_of(_hwnd: isize) -> [f32; 4] {
    work_area_at_cursor()
}

pub fn allow_any_foreground() {}

pub fn set_topmost(_hwnd: isize, _on: bool) {}

pub fn set_rect(_hwnd: isize, _rect: [f32; 4], _scale: f32) {}

pub fn set_show_in_taskbar(_hwnd: isize, _show: bool) {}

/// Whether a process with this executable name runs (for the current user or anyone).
pub fn process_running(exe: &str) -> bool {
    let want = exe.trim_end_matches(".exe");
    let Ok(rd) = std::fs::read_dir("/proc") else { return false };
    rd.flatten().filter(|e| e.file_name().to_string_lossy().bytes().all(|b| b.is_ascii_digit())).any(|e| {
        // comm is cut at 15 characters
        std::fs::read_to_string(e.path().join("comm")).is_ok_and(|c| {
            let c = c.trim_end();
            c == want || (c.len() == 15 && want.starts_with(c))
        })
    })
}
