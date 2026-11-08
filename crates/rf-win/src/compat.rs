//! Windows types that appear in this crate's API (window handles, rectangles, GUIDs, drop
//! effects). On Windows they are the real `windows` crate types; elsewhere plain look-alikes,
//! so callers compile unchanged on every platform.

#[cfg(windows)]
pub use windows::Win32::Foundation::{HWND, POINT, RECT};
#[cfg(windows)]
pub use windows::Win32::System::Ole::DROPEFFECT;
#[cfg(windows)]
pub use windows::core::GUID;

#[cfg(not(windows))]
pub use imp::*;

#[cfg(not(windows))]
mod imp {
    /// A native window handle (an opaque number; no window system call takes it here).
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct HWND(pub isize);

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct GUID(pub u128);

    /// What a drop did (same bit values as Windows' DROPEFFECT_*).
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct DROPEFFECT(pub u32);

    impl DROPEFFECT {
        pub const NONE: DROPEFFECT = DROPEFFECT(0);
        pub const COPY: DROPEFFECT = DROPEFFECT(1);
        pub const MOVE: DROPEFFECT = DROPEFFECT(2);
        pub const LINK: DROPEFFECT = DROPEFFECT(4);
    }
}
