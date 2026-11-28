//! Which programs hold a file open: Windows' Restart Manager, the same service installers ask before they replace a file. It needs
//! no administrator rights and sees the programs of every user that hold one of the files (a Discord request, 2026-11-28: "see what's
//! locking your files or folders").
//!
//! A folder is asked about through the files in it (Restart Manager knows files, not folders): the first [`MAX_FILES`] of them, a
//! few levels deep, which is where the file a program has open usually is.

use std::path::{Path, PathBuf};

use windows::Win32::Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS};
use windows::Win32::System::RestartManager::{CCH_RM_SESSION_KEY, RM_PROCESS_INFO, RmEndSession, RmGetList, RmRegisterResources, RmStartSession};
use windows::core::{PCWSTR, PWSTR};

use crate::wide::wide;

/// At most this many files of a folder are asked about.
pub const MAX_FILES: usize = 2000;
/// ... found no deeper than this under the folder.
const MAX_DEPTH: usize = 4;

/// A program that holds one of the files open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locker {
    pub pid: u32,
    /// The program's name as Windows tells it ("Microsoft Word", "notepad.exe").
    pub name: String,
}

/// The programs that hold `paths` (files, or folders through their files) open. Blocking: off the UI thread.
pub fn who_holds(paths: &[PathBuf]) -> anyhow::Result<Vec<Locker>> {
    let files = files_of(paths);
    if files.is_empty() {
        return Ok(Vec::new());
    }
    let mut session = 0u32;
    let mut key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
    let r = unsafe { RmStartSession(&mut session, None, PWSTR(key.as_mut_ptr())) };
    anyhow::ensure!(r == ERROR_SUCCESS, "Restart Manager did not start: {}", r.0);
    let out = list(session, &files);
    unsafe {
        let _ = RmEndSession(session);
    }
    out
}

fn list(session: u32, files: &[PathBuf]) -> anyhow::Result<Vec<Locker>> {
    let wides: Vec<Vec<u16>> = files.iter().map(|f| wide(f)).collect();
    let names: Vec<PCWSTR> = wides.iter().map(|w| PCWSTR(w.as_ptr())).collect();
    // (registered in parts: one call with thousands of names is refused on some systems)
    for part in names.chunks(256) {
        let r = unsafe { RmRegisterResources(session, Some(part), None, None) };
        anyhow::ensure!(r == ERROR_SUCCESS, "Restart Manager did not take the files: {}", r.0);
    }
    let mut infos: Vec<RM_PROCESS_INFO> = Vec::new();
    for _ in 0..4 {
        let (mut needed, mut count, mut reasons) = (0u32, infos.len() as u32, 0u32);
        let ptr = if infos.is_empty() { None } else { Some(infos.as_mut_ptr()) };
        let r = unsafe { RmGetList(session, &mut needed, &mut count, ptr, &mut reasons) };
        if r == ERROR_SUCCESS {
            infos.truncate(count as usize);
            return Ok(lockers_of(&infos));
        }
        anyhow::ensure!(r == ERROR_MORE_DATA, "Restart Manager gave no list: {}", r.0);
        // (programs may open the files meanwhile: room for a few more)
        infos = vec![RM_PROCESS_INFO::default(); needed as usize + 4];
    }
    anyhow::bail!("the list of programs kept changing")
}

fn lockers_of(infos: &[RM_PROCESS_INFO]) -> Vec<Locker> {
    let mut out: Vec<Locker> = Vec::new();
    for i in infos {
        let n = i.strAppName.iter().position(|&c| c == 0).unwrap_or(i.strAppName.len());
        let name = String::from_utf16_lossy(&i.strAppName[..n]);
        let pid = i.Process.dwProcessId;
        if !out.iter().any(|l| l.pid == pid) {
            out.push(Locker { pid, name });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// The files to ask about: each file as it is, a folder's files (see the module).
fn files_of(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for p in paths {
        if p.is_dir() {
            walk(p, 0, &mut out);
        } else {
            out.push(p.clone());
        }
        if out.len() >= MAX_FILES {
            out.truncate(MAX_FILES);
            break;
        }
    }
    out
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut sub = Vec::new();
    for e in rd.flatten() {
        if out.len() >= MAX_FILES {
            return;
        }
        let Ok(t) = e.file_type() else { continue };
        if t.is_dir() {
            // (not into links: a junction can lead anywhere, even back up)
            if !t.is_symlink() {
                sub.push(e.path());
            }
        } else {
            out.push(e.path());
        }
    }
    if depth < MAX_DEPTH {
        for d in sub {
            walk(&d, depth + 1, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    /// A file this test holds open is reported with this process; once closed, nothing holds it.
    #[test]
    fn a_file_held_open_names_the_program_that_holds_it() {
        let dir = std::env::temp_dir().join(format!("rf-lockers-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("held.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"x").unwrap();
        let held = super::who_holds(&[path.clone()]).unwrap();
        assert!(held.iter().any(|l| l.pid == std::process::id()), "{held:?}");
        drop(f);
        let free = super::who_holds(&[path.clone()]).unwrap();
        assert!(!free.iter().any(|l| l.pid == std::process::id()), "{free:?}");
        // a folder is asked about through its files
        let f2 = std::fs::File::open(&path).unwrap();
        let by_folder = super::who_holds(&[dir.clone()]).unwrap();
        assert!(by_folder.iter().any(|l| l.pid == std::process::id()), "{by_folder:?}");
        drop(f2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
