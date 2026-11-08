//! Indexed search (Windows Search). TODO(linux, phase 6): optional `plocate` (names) and
//! Baloo / LocalSearch (contents) backends. Without an index the app falls back to walking.

use std::path::Path;

use super::search::SearchHit;

pub fn is_indexed(_root: &Path) -> bool {
    false
}

pub fn search(
    _root: &Path,
    _query: &str,
    _max_results: usize,
    _cancelled: &dyn Fn() -> bool,
    _on_batch: &mut dyn FnMut(Vec<SearchHit>),
) -> anyhow::Result<usize> {
    Err(super::unsupported("indexed search"))
}
