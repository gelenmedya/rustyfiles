//! Our own copy / move engine. TODO(linux, phase 3): port it with `copy_file_range`, reflinks
//! (FICLONE) and `renameat2`; keep mode, times and xattrs. The types are the real ones (the
//! conflict dialog and settings use them); the engine itself reports "unsupported" for now.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use super::fileops::{Tracker, Transfer};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Verify {
    #[default]
    Off,
    /// Same size and modification time.
    Quick,
    /// Same bytes, read back from the disk (not from the cache).
    Full,
}

/// A name that already exists at the destination.
#[derive(Clone, Debug)]
pub struct Conflict {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub src_dir: bool,
    pub dst_dir: bool,
    pub src_size: u64,
    pub dst_size: u64,
    pub src_time: Option<SystemTime>,
    pub dst_time: Option<SystemTime>,
}

impl Conflict {
    /// A file against a folder (or the other way round): it can't replace, only skip or keep both.
    pub fn mismatched(&self) -> bool {
        self.src_dir != self.dst_dir
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resolve {
    Replace,
    Skip,
    KeepBoth,
}

pub const NOT_VERIFIED: &str = "Doğrulanamayan dosyalar";
pub const VERIFYING: &str = "Doğrulanıyor…";

/// Copy or move `sources` into `dest_dir` with progress. TODO(linux, phase 3).
pub fn transfer(kind: Transfer, sources: &[PathBuf], dest_dir: &Path, tracker: Arc<Tracker>) -> anyhow::Result<bool> {
    super::fileops::transfer_tracked(kind, sources, dest_dir, tracker)
}
