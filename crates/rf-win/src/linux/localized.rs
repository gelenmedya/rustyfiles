//! Localized folder names (Windows' desktop.ini). Linux folders carry their real names.

use std::path::Path;
use std::sync::Arc;

pub fn folder_label(_dir: &Path, _attrs: Option<u32>) -> Option<Arc<str>> {
    None
}

/// Whether `p` is on a network filesystem (NFS, SMB, sshfs…).
pub fn is_network(p: &Path) -> bool {
    super::drives::list_drives()
        .into_iter()
        .filter(|d| p.starts_with(&d.root))
        .max_by_key(|d| d.root.as_os_str().len())
        .is_some_and(|d| d.kind == super::drives::DriveKind::Network)
}
