//! File operations, Linux version. Windows runs these through the shell copy engine; Linux has
//! none, so phase 3 of the port builds them on our own engine (`fastcopy`) and the freedesktop
//! trash. Until then copy / move / delete report "unsupported" — deliberately: a half-built
//! copy engine is how data gets lost. Rename and new items work.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transfer {
    Copy,
    Move,
}

/// Copy or move into `dest_dir`. TODO(linux, phase 3).
pub fn transfer(kind: Transfer, sources: &[PathBuf], dest_dir: &Path) -> anyhow::Result<bool> {
    let same_dir = sources.iter().all(|s| s.parent() == Some(dest_dir));
    if kind == Transfer::Move && same_dir {
        return Ok(true); // moving onto itself: no-op
    }
    Err(super::unsupported("copy / move"))
}

/// Delete to the trash (`permanent = false`) or permanently. TODO(linux, phase 3).
pub fn delete(_paths: &[PathBuf], _permanent: bool) -> anyhow::Result<bool> {
    Err(super::unsupported("delete"))
}

/// Renames `path` within its folder. Refuses to replace an existing item.
pub fn rename(path: &Path, new_name: &str) -> anyhow::Result<bool> {
    anyhow::ensure!(!new_name.is_empty() && !new_name.contains('/') && new_name != "." && new_name != "..", "Geçersiz ad");
    let to = path.with_file_name(new_name);
    if to == path {
        return Ok(true);
    }
    rename_no_replace(path, &to)?;
    Ok(true)
}

/// Plain rename that fails instead of silently replacing an existing target
/// (`renameat2(RENAME_NOREPLACE)`; `std::fs::rename` replaces).
pub fn rename_no_replace(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let a = std::ffi::CString::new(from.as_os_str().as_bytes())?;
    let b = std::ffi::CString::new(to.as_os_str().as_bytes())?;
    let r = unsafe { libc::renameat2(libc::AT_FDCWD, a.as_ptr(), libc::AT_FDCWD, b.as_ptr(), libc::RENAME_NOREPLACE) };
    if r == 0 {
        return Ok(());
    }
    let err = std::io::Error::last_os_error();
    // filesystems without RENAME_NOREPLACE (some FUSE / network ones): check, then rename
    if matches!(err.raw_os_error(), Some(libc::EINVAL) | Some(libc::ENOSYS)) {
        if std::fs::symlink_metadata(to).is_ok() {
            return Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
        }
        return std::fs::rename(from, to);
    }
    Err(err)
}

/// "Yeni klasör", "Yeni klasör (2)", … — returns the created path.
pub fn new_folder(dir: &Path) -> anyhow::Result<PathBuf> {
    let p = unique_name(dir, "Yeni klasör", "");
    std::fs::create_dir(&p)?;
    Ok(p)
}

pub fn new_text_file(dir: &Path) -> anyhow::Result<PathBuf> {
    let p = unique_name(dir, "Yeni Metin Belgesi", ".txt");
    std::fs::OpenOptions::new().write(true).create_new(true).open(&p)?;
    Ok(p)
}

pub fn unique_name(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let first = dir.join(format!("{stem}{ext}"));
    if std::fs::symlink_metadata(&first).is_err() {
        return first;
    }
    (2..)
        .map(|i| dir.join(format!("{stem} ({i}){ext}")))
        .find(|p| std::fs::symlink_metadata(p).is_err())
        .unwrap()
}

/// Progress reporting for one operation (same as Windows). `on_progress(total, done, current_item)`.
pub struct Tracker {
    pub on_progress: Box<dyn Fn(u32, u32, Option<String>) + Send + Sync>,
    pub cancel: Arc<AtomicBool>,
    /// (source, result) for every finished item; used to build undo records.
    pub results: std::sync::Mutex<Vec<(PathBuf, PathBuf)>>,
    /// Our engine waits while this is set.
    pub paused: Arc<AtomicBool>,
    /// Copy speed limit in bytes per second; 0 = none.
    pub limit: std::sync::atomic::AtomicU64,
    /// Check copies after writing them; see [`Tracker::set_verify`].
    pub verify: std::sync::atomic::AtomicU8,
    /// Asked (on the worker thread, blocking) what to do with names that already exist at the
    /// destination; `None` cancels the operation. Without it, both are kept.
    #[allow(clippy::type_complexity)]
    pub ask: Option<Box<dyn Fn(&[super::fastcopy::Conflict]) -> Option<Vec<super::fastcopy::Resolve>> + Send + Sync>>,
}

impl Tracker {
    pub fn set_verify(&self, v: super::fastcopy::Verify) {
        self.verify.store(v as u8, Ordering::Relaxed);
    }

    pub fn verify_mode(&self) -> super::fastcopy::Verify {
        match self.verify.load(Ordering::Relaxed) {
            1 => super::fastcopy::Verify::Quick,
            2 => super::fastcopy::Verify::Full,
            _ => super::fastcopy::Verify::Off,
        }
    }

    pub fn new(on_progress: Box<dyn Fn(u32, u32, Option<String>) + Send + Sync>, cancel: Arc<AtomicBool>) -> Self {
        Self {
            on_progress,
            cancel,
            results: Default::default(),
            paused: Default::default(),
            limit: Default::default(),
            verify: Default::default(),
            ask: None,
        }
    }
}

/// `transfer` with in-app progress. TODO(linux, phase 3).
pub fn transfer_tracked(kind: Transfer, sources: &[PathBuf], dest_dir: &Path, _tracker: Arc<Tracker>) -> anyhow::Result<bool> {
    transfer(kind, sources, dest_dir)
}

/// `delete` with in-app progress. TODO(linux, phase 3).
pub fn delete_tracked(paths: &[PathBuf], permanent: bool, _tracker: Arc<Tracker>) -> anyhow::Result<bool> {
    delete(paths, permanent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_and_new_items() {
        let d = std::env::temp_dir().join(format!("rf-ops-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let a = new_folder(&d).unwrap();
        let b = new_folder(&d).unwrap();
        assert!(a.ends_with("Yeni klasör") && b.ends_with("Yeni klasör (2)"));
        let f = new_text_file(&d).unwrap();
        assert!(rename(&f, "x.txt").unwrap());
        assert!(d.join("x.txt").exists());
        // never replaces
        std::fs::write(d.join("y.txt"), b"keep").unwrap();
        assert!(rename(&d.join("x.txt"), "y.txt").is_err());
        assert_eq!(std::fs::read(d.join("y.txt")).unwrap(), b"keep");
        std::fs::remove_dir_all(&d).unwrap();
    }
}
