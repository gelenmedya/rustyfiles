//! Recursive filename search ("in this folder and subfolders"), Linux version.
//!
//! Same design as the Windows module: directories are walked in parallel on a dedicated rayon
//! pool at low priority, results stream in batches to the calling thread. Only the directory
//! reading differs (`read_dir` instead of FindFirstFileExW).
//!
//! TODO(linux, phase 2): share the matcher and walker with the Windows module instead of
//! keeping two copies.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use crossbeam_channel::{RecvTimeoutError, Sender, unbounded};
use rf_core::{Entry, attrs};

use super::listing::to_entry;

const BATCH: usize = 200;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub root: PathBuf,
    /// Case-insensitive substring, or a glob when it contains `*` / `?` (matched against the whole name).
    pub pattern: String,
    pub include_hidden: bool,
    pub max_results: usize,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub path: PathBuf,
    pub entry: Entry,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchStats {
    pub dirs_scanned: usize,
    pub entries_scanned: usize,
    pub hits: usize,
    /// Directories that could not be read (access denied etc.).
    pub skipped_dirs: usize,
    /// Stopped because `max_results` was reached.
    pub truncated: bool,
    pub cancelled: bool,
}

fn fold(s: &str) -> Vec<char> {
    s.to_lowercase().chars().filter(|&c| c != '\u{307}').collect()
}

enum Matcher {
    All,
    Substr(Vec<char>),
    Glob(Vec<char>),
}

impl Matcher {
    fn new(pattern: &str) -> Self {
        let p = fold(pattern.trim());
        if p.is_empty() {
            Matcher::All
        } else if p.iter().any(|&c| c == '*' || c == '?') {
            Matcher::Glob(p)
        } else {
            Matcher::Substr(p)
        }
    }

    fn is_match(&self, name: &str) -> bool {
        match self {
            Matcher::All => true,
            Matcher::Substr(p) => {
                let n = fold(name);
                n.len() >= p.len() && n.windows(p.len()).any(|w| w == p.as_slice())
            }
            Matcher::Glob(p) => glob_match(p, &fold(name)),
        }
    }
}

fn glob_match(p: &[char], n: &[char]) -> bool {
    let (mut pi, mut ni) = (0, 0);
    let (mut star, mut mark) = (usize::MAX, 0);
    while ni < n.len() {
        if pi < p.len() && p[pi] == '*' {
            star = pi;
            mark = ni;
            pi += 1;
        } else if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if star != usize::MAX {
            pi = star + 1;
            mark += 1;
            ni = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Enumerates `dir`, calling `f` per entry. Err = errno.
pub(crate) fn for_each_raw(dir: &Path, mut f: impl FnMut(&std::fs::DirEntry)) -> Result<(), u32> {
    let rd = std::fs::read_dir(dir).map_err(|e| e.raw_os_error().unwrap_or(0) as u32)?;
    for de in rd.flatten() {
        f(&de);
    }
    Ok(())
}

/// Lowers this thread's CPU priority (Linux `nice` is per thread).
pub(crate) fn lower_thread_priority(nice: i32) {
    unsafe {
        let _ = libc::setpriority(libc::PRIO_PROCESS, 0, nice);
    }
}

struct Shared<'a> {
    matcher: &'a Matcher,
    include_hidden: bool,
    max: usize,
    stop: &'a AtomicBool,
    reserved: &'a AtomicUsize,
    dirs: &'a AtomicUsize,
    entries: &'a AtomicUsize,
    skipped: &'a AtomicUsize,
    tx: &'a Sender<Vec<SearchHit>>,
}

fn walk<'s>(sc: &rayon::Scope<'s>, sh: &'s Shared<'s>, dir: PathBuf) {
    if sh.stop.load(Ordering::Relaxed) {
        return;
    }
    let mut batch: Vec<SearchHit> = Vec::new();
    let mut subdirs: Vec<PathBuf> = Vec::new();
    let mut count = 0usize;
    let r = for_each_raw(&dir, |d| {
        if sh.stop.load(Ordering::Relaxed) {
            return;
        }
        let Some(entry) = to_entry(d) else { return };
        count += 1;
        let a = entry.attrs;
        if a & attrs::HIDDEN != 0 && !sh.include_hidden {
            return;
        }
        if entry.is_dir() && a & attrs::REPARSE_POINT == 0 {
            subdirs.push(dir.join(&*entry.name));
        }
        if sh.matcher.is_match(&entry.name) {
            if sh.reserved.fetch_add(1, Ordering::Relaxed) >= sh.max {
                sh.stop.store(true, Ordering::Relaxed);
                return;
            }
            batch.push(SearchHit { path: dir.join(&*entry.name), entry });
            if batch.len() >= BATCH {
                let _ = sh.tx.send(std::mem::take(&mut batch));
            }
        }
    });
    sh.dirs.fetch_add(1, Ordering::Relaxed);
    sh.entries.fetch_add(count, Ordering::Relaxed);
    if let Err(code) = r {
        if code != libc::EACCES as u32 {
            log::debug!("search: {} failed ({code})", dir.display());
        }
        sh.skipped.fetch_add(1, Ordering::Relaxed);
    }
    if !batch.is_empty() {
        let _ = sh.tx.send(batch);
    }
    for sub in subdirs {
        sc.spawn(move |sc| walk(sc, sh, sub));
    }
}

/// Searches `q.root` recursively; see the Windows module for the full contract.
pub fn search(
    q: &SearchQuery,
    cancelled: &dyn Fn() -> bool,
    on_batch: &mut dyn FnMut(Vec<SearchHit>),
) -> anyhow::Result<SearchStats> {
    anyhow::ensure!(q.root.is_dir(), "Klasör bulunamadı: {}", q.root.display());
    let matcher = Matcher::new(&q.pattern);
    let max = q.max_results;
    let stop = AtomicBool::new(false);
    let reserved = AtomicUsize::new(0);
    let (dirs, entries, skipped) = (AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0));
    let (tx, rx) = unbounded::<Vec<SearchHit>>();

    let threads = std::thread::available_parallelism().map_or(2, |n| (n.get() / 2).clamp(2, 6));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|i| format!("rf-search-{i}"))
        .start_handler(|_| lower_thread_priority(10))
        .build()?;

    let shared = Shared {
        matcher: &matcher,
        include_hidden: q.include_hidden,
        max,
        stop: &stop,
        reserved: &reserved,
        dirs: &dirs,
        entries: &entries,
        skipped: &skipped,
        tx: &tx,
    };
    let mut hits = 0usize;
    let mut was_cancelled = false;
    let mut emit = |mut b: Vec<SearchHit>, was_cancelled: bool| {
        if was_cancelled || hits >= max {
            return;
        }
        b.truncate(max - hits);
        hits += b.len();
        on_batch(b);
    };
    std::thread::scope(|ts| {
        let walker = ts.spawn(|| {
            if max > 0 {
                let sh = &shared;
                pool.scope(|sc| walk(sc, sh, q.root.clone()));
            }
        });
        loop {
            if !was_cancelled && cancelled() {
                was_cancelled = true;
                stop.store(true, Ordering::Relaxed);
            }
            match rx.recv_timeout(Duration::from_millis(10)) {
                Ok(b) => emit(b, was_cancelled),
                Err(RecvTimeoutError::Timeout) => {
                    if walker.is_finished() {
                        while let Ok(b) = rx.try_recv() {
                            emit(b, was_cancelled);
                        }
                        break;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        let _ = walker.join();
    });

    Ok(SearchStats {
        dirs_scanned: dirs.load(Ordering::Relaxed),
        entries_scanned: entries.load(Ordering::Relaxed),
        hits,
        skipped_dirs: skipped.load(Ordering::Relaxed),
        truncated: !was_cancelled && reserved.load(Ordering::Relaxed) > max,
        cancelled: was_cancelled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substring_and_glob() {
        let root = std::env::temp_dir().join(format!("rf-search-lx-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("a/b")).unwrap();
        for f in ["Report.TXT", "a/report2.txt", "a/b/notes.md", "a/.secret_report"] {
            std::fs::write(root.join(f), b"x").unwrap();
        }
        let run = |pat: &str| {
            let q = SearchQuery { root: root.clone(), pattern: pat.into(), include_hidden: false, max_results: 100 };
            let mut names = Vec::new();
            search(&q, &|| false, &mut |b| names.extend(b.into_iter().map(|h| h.entry.name.to_string()))).unwrap();
            names.sort();
            names
        };
        assert_eq!(run("REPORT"), vec!["Report.TXT", "report2.txt"]);
        assert_eq!(run("*.md"), vec!["notes.md"]);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
