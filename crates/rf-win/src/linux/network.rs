//! Network locations. TODO(linux, phase 6): SMB / SFTP / WebDAV through gvfs (`gio mount`,
//! browsing under /run/user/$UID/gvfs). Windows' `\\server` roots don't exist here.

use std::path::{Path, PathBuf};

pub fn server_root(_path: &Path) -> Option<String> {
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    pub name: String,
    pub remark: String,
}

pub fn shares(_server: &str) -> std::io::Result<Vec<Share>> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "network shares are not available on Linux yet"))
}

pub fn network_shortcuts() -> Vec<(String, PathBuf)> {
    Vec::new()
}

pub fn map_drive_dialog(_owner: isize) {}

pub fn disconnect_drive_dialog(_owner: isize) {}
