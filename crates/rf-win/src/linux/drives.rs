//! "Drives" on Linux: the root filesystem plus the user-visible mounts (USB sticks and discs
//! under /media, /run/media, /mnt; network shares). Read from /proc/self/mounts.
//!
//! TODO(linux, phase 6): UDisks2 for labels, unmounted devices and eject.

use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveKind {
    Fixed,
    Removable,
    Network,
    Optical,
    RamDisk,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Drive {
    /// The mount point ("/", "/run/media/me/USB").
    pub root: PathBuf,
    /// Windows' drive letter. Linux has none: a stable placeholder ('/' for the root, else '·').
    pub letter: char,
    pub label: String,
    pub kind: DriveKind,
    pub total: u64,
    pub free: u64,
    pub fs: String,
}

impl Drive {
    pub fn display_name(&self) -> String {
        if !self.label.is_empty() {
            return self.label.clone();
        }
        match self.kind {
            DriveKind::Removable => "USB Sürücü",
            DriveKind::Network => "Ağ Sürücüsü",
            DriveKind::Optical => "DVD Sürücü",
            _ => "Yerel Disk",
        }
        .to_string()
    }
    pub fn used_fraction(&self) -> f32 {
        if self.total == 0 { 0.0 } else { 1.0 - self.free as f32 / self.total as f32 }
    }
}

struct Mount {
    device: String,
    point: PathBuf,
    fs: String,
}

/// `\040` style escapes in /proc/self/mounts fields.
fn unescape(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 3 < b.len() && b[i + 1..i + 4].iter().all(|c| (b'0'..=b'7').contains(c)) {
            out.push((b[i + 1] - b'0') * 64 + (b[i + 2] - b'0') * 8 + (b[i + 3] - b'0'));
            i += 4;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn mounts() -> Vec<Mount> {
    let Ok(t) = std::fs::read_to_string("/proc/self/mounts") else { return Vec::new() };
    t.lines()
        .filter_map(|l| {
            let mut f = l.split(' ');
            let device = unescape(f.next()?);
            let point = PathBuf::from(unescape(f.next()?));
            let fs = f.next()?.to_string();
            Some(Mount { device, point, fs })
        })
        .collect()
}

const NETWORK_FS: &[&str] = &["nfs", "nfs4", "cifs", "smb3", "smbfs", "sshfs", "fuse.sshfs", "fuse.rclone", "davfs", "9p"];

/// Mounts a user would call a drive: `/`, and real filesystems under the usual media folders.
fn visible(m: &Mount) -> Option<DriveKind> {
    let p = m.point.as_os_str().as_bytes();
    if p == b"/" {
        return Some(DriveKind::Fixed);
    }
    if NETWORK_FS.contains(&m.fs.as_str()) {
        return Some(DriveKind::Network);
    }
    let media = p.starts_with(b"/media/") || p.starts_with(b"/run/media/") || p.starts_with(b"/mnt/");
    if !media || !m.device.starts_with("/dev/") {
        return None;
    }
    if m.fs == "iso9660" || m.fs == "udf" {
        return Some(DriveKind::Optical);
    }
    Some(DriveKind::Removable)
}

fn space(root: &Path) -> (u64, u64) {
    let Ok(c) = std::ffi::CString::new(root.as_os_str().as_bytes()) else { return (0, 0) };
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return (0, 0);
    }
    let unit = st.f_frsize as u64;
    (st.f_blocks as u64 * unit, st.f_bavail as u64 * unit)
}

/// Lists the visible mounts. `statvfs` can stall on a dead network mount: off the UI thread.
pub fn list_drives() -> Vec<Drive> {
    let mut out: Vec<Drive> = Vec::new();
    for m in mounts() {
        let Some(kind) = visible(&m) else { continue };
        if out.iter().any(|d| d.root == m.point) {
            continue;
        }
        let (total, free) = space(&m.point);
        let label = if m.point.as_os_str() == "/" {
            "Dosya Sistemi".to_string()
        } else {
            m.point.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
        };
        let letter = if m.point.as_os_str() == "/" { '/' } else { '·' };
        out.push(Drive { root: m.point, letter, label, kind, total, free, fs: m.fs });
    }
    out
}

/// Whether `path` is on a removable drive (USB stick, SD card).
pub fn is_removable(path: &Path) -> bool {
    list_drives()
        .into_iter()
        .filter(|d| path.starts_with(&d.root))
        .max_by_key(|d| d.root.as_os_str().len())
        .is_some_and(|d| d.kind == DriveKind::Removable)
}

/// Changes whenever the set of visible mounts changes (cheap; the sidebar polls it).
pub fn drive_mask() -> u32 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for m in mounts() {
        if visible(&m).is_some() {
            m.point.hash(&mut h);
        }
    }
    h.finish() as u32
}

/// Whether the disk holding `path` spins (`/sys/block/<dev>/queue/rotational`).
/// TODO(linux): map the mount's device to its block device; unknown for now.
pub fn seek_penalty(_path: &Path) -> Option<bool> {
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn has_root() {
        let d = super::list_drives();
        assert!(d.iter().any(|d| d.root == std::path::Path::new("/")));
        assert_eq!(super::unescape(r"/media/My\040Disk"), "/media/My Disk");
    }
}
