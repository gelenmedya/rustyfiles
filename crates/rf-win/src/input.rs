//! Global input state the UI asks for outside GPUI's events: the mouse position on screen,
//! whether Shift / Ctrl are held right now, the double-click time.
//!
//! On Linux (Wayland) an app can't read the global pointer or keyboard state, so these return
//! neutral values; callers should prefer what GPUI reports with the event.

use std::time::Duration;

/// Mouse position in screen pixels.
pub fn cursor_pos() -> (i32, i32) {
    #[cfg(windows)]
    {
        let mut pt = windows::Win32::Foundation::POINT::default();
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt);
        }
        (pt.x, pt.y)
    }
    #[cfg(not(windows))]
    {
        (0, 0)
    }
}

/// Shift is held down right now.
pub fn shift_down() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_SHIFT};
        unsafe { GetKeyState(VK_SHIFT.0 as i32) < 0 }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Ctrl is held down right now.
pub fn ctrl_down() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL};
        unsafe { GetKeyState(VK_CONTROL.0 as i32) < 0 }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// The system double-click interval.
pub fn double_click_time() -> Duration {
    #[cfg(windows)]
    {
        Duration::from_millis(unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime() } as u64)
    }
    #[cfg(not(windows))]
    {
        // GTK's and Qt's default
        Duration::from_millis(400)
    }
}

/// Id of the calling thread, as the drag-and-drop code wants it.
pub fn current_thread_id() -> u32 {
    #[cfg(windows)]
    {
        unsafe { windows::Win32::System::Threading::GetCurrentThreadId() }
    }
    #[cfg(not(windows))]
    {
        0
    }
}

/// Lets go of the mouse capture GPUI took for a press (before a native drag loop starts).
pub fn release_mouse_capture() {
    #[cfg(windows)]
    unsafe {
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
    }
}
