//! Parallel permanent delete (Shift+Delete). TODO(linux, phase 3): port with `unlinkat`.

use std::path::PathBuf;
use std::sync::Arc;

use super::fileops::Tracker;

pub fn delete_permanent(_paths: &[PathBuf], _tracker: Arc<Tracker>, _threads: usize) -> anyhow::Result<bool> {
    Err(super::unsupported("permanent delete"))
}

pub fn remove_verified(_files: &[PathBuf], _dirs: &[PathBuf]) -> bool {
    false
}

pub fn threads_for(paths: &[PathBuf]) -> usize {
    paths.len().clamp(1, 8)
}
