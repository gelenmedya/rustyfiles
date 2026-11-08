//! Taskbar jump list. Linux equivalent: `Actions=` in the installed `.desktop` file (static,
//! written by the package), so there is nothing to update at run time.

use std::path::{Path, PathBuf};

pub const NEW_WINDOW_ARG: &str = "--new-window";

pub fn set(_exe: &Path, _tasks: &[(&str, &str)], _pinned_title: &str, _pinned: &[(String, PathBuf)]) -> anyhow::Result<()> {
    Ok(())
}
