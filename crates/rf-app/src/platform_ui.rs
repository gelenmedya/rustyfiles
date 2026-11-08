//! The extras our GPUI fork for Windows (`gpui-pre-windows`) exposes beyond GPUI itself: the
//! hybrid GPU/CPU renderer, "keep the window looking active" while a flyout is open, smooth
//! scrolling and the drag-and-drop cursor hint.
//!
//! On Windows these are the fork's own items. Elsewhere GPUI's stock backend runs (wgpu on
//! Linux): one renderer, no switching, and the hints have nothing to talk to.

#[cfg(windows)]
pub use gpui_windows::{KEEP_ACTIVE_LOOK, SMOOTH_SCROLL, dnd_hint, hybrid};

#[cfg(not(windows))]
pub use imp::*;

#[cfg(not(windows))]
mod imp {
    use std::sync::atomic::AtomicBool;

    /// Keep the main window drawn as active while one of our popups has focus.
    pub static KEEP_ACTIVE_LOOK: AtomicBool = AtomicBool::new(false);
    /// Smooth (animated) wheel scrolling; GPUI's Linux backend scrolls as the compositor reports.
    pub static SMOOTH_SCROLL: AtomicBool = AtomicBool::new(true);

    /// The cursor hint during an in-app drag (copy / move).
    pub mod dnd_hint {
        pub const COPY: u32 = 1;
        pub const MOVE: u32 = 2;
        pub fn set(_hint: u32) {}
    }

    /// The hybrid renderer's controls. Linux has the one GPUI renderer (wgpu).
    pub mod hybrid {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum RendererMode {
            Auto,
            Gpu,
            Cpu,
        }

        #[derive(Debug, Clone, Copy, Default)]
        pub struct LoadSnapshot {
            pub gpu_percent: Option<f32>,
            pub cpu_percent: Option<f32>,
            pub last_frame_ms: f32,
        }

        pub fn mode() -> RendererMode {
            RendererMode::Auto
        }

        pub fn set_mode(_mode: RendererMode) {}

        pub fn switch_count() -> u64 {
            0
        }

        pub fn current_backend_label() -> &'static str {
            "wgpu"
        }

        pub fn load_snapshot() -> LoadSnapshot {
            LoadSnapshot::default()
        }
    }
}
