//! Windows `.lnk` shortcuts. TODO(linux, phase 5): symlinks and `.desktop` launchers.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShortcutInfo {
    /// Empty when the target is not a file-system path.
    pub target: PathBuf,
    pub args: String,
    pub working_dir: PathBuf,
    /// Icon file ("" = use the target's icon) and its resource index.
    pub icon: String,
    pub icon_index: i32,
}

pub fn resolve(_path: &Path) -> anyhow::Result<ShortcutInfo> {
    Err(super::unsupported(".lnk shortcuts"))
}

pub fn create(_link_path: &Path, _target: &Path) -> anyhow::Result<()> {
    Err(super::unsupported("creating shortcuts"))
}
