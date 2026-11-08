//! Starting helper processes (git, rar…) without a console window flashing up.

use std::process::Command;

/// `CREATE_NO_WINDOW`: a console program started from a GUI app gets no window of its own.
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub trait NoWindow {
    /// Windows: sets these process creation flags (normally [`CREATE_NO_WINDOW`]).
    /// Elsewhere there is no console window to hide: does nothing.
    fn no_window(&mut self, flags: u32) -> &mut Self;
}

impl NoWindow for Command {
    fn no_window(&mut self, flags: u32) -> &mut Self {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            self.creation_flags(flags)
        }
        #[cfg(not(windows))]
        {
            let _ = flags;
            self
        }
    }
}
