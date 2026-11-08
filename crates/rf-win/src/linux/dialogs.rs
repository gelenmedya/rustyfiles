//! File pickers and "start with the system".
//!
//! Pickers: `zenity` / `kdialog` when installed. TODO(linux, phase 5): the xdg-desktop-portal
//! FileChooser (`ashpd`), which every desktop provides.
//! Autostart: an XDG autostart entry (`~/.config/autostart/<name>.desktop`).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::compat::HWND;

pub enum Pick<'a> {
    /// Open an existing file; filters are (label, pattern) pairs like ("Ayar dosyası", "*.toml").
    File { filters: &'a [(&'a str, &'a str)] },
    /// Choose a folder.
    Folder,
    /// Save as `default_name`.
    Save { filters: &'a [(&'a str, &'a str)], default_name: &'a str },
}

fn run(exe: &str, args: &[String]) -> Option<Option<PathBuf>> {
    let out = Command::new(exe).args(args).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    if !out.status.success() {
        return Some(None); // cancelled
    }
    let s = String::from_utf8_lossy(&out.stdout).trim_end_matches('\n').to_string();
    Some((!s.is_empty()).then(|| PathBuf::from(s)))
}

/// Shows a modal file / folder picker; `None` when cancelled or no picker tool is installed.
pub fn pick(_owner: HWND, title: &str, kind: Pick<'_>) -> Option<PathBuf> {
    let mut z = vec!["--file-selection".to_string(), format!("--title={title}")];
    let mut k: Vec<String> = vec!["--title".into(), title.into()];
    match &kind {
        Pick::File { filters } => {
            for (label, pat) in filters.iter() {
                z.push(format!("--file-filter={label} | {}", pat.replace(';', " ")));
            }
            k.extend(["--getopenfilename".into(), ".".into()]);
            if let Some((label, pat)) = filters.first() {
                k.push(format!("{label} ({})", pat.replace(';', " ")));
            }
        }
        Pick::Folder => {
            z.push("--directory".into());
            k.extend(["--getexistingdirectory".into(), ".".into()]);
        }
        Pick::Save { filters, default_name } => {
            z.extend(["--save".into(), "--confirm-overwrite".into(), format!("--filename={default_name}")]);
            for (label, pat) in filters.iter() {
                z.push(format!("--file-filter={label} | {}", pat.replace(';', " ")));
            }
            k.extend(["--getsavefilename".into(), default_name.to_string()]);
        }
    }
    run("zenity", &z).or_else(|| run("kdialog", &k)).flatten()
}

fn autostart_file(name: &str) -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("autostart").join(format!("{name}.desktop")))
}

pub fn starts_with_windows() -> bool {
    starts_with_windows_as("rustyfiles")
}

pub fn starts_with_windows_as(name: &str) -> bool {
    autostart_file(name).is_some_and(|p| p.is_file())
}

pub fn set_start_with_windows(exe: &Path, on: bool) -> std::io::Result<()> {
    set_start_with_windows_as("rustyfiles", exe, &["--background"], on)
}

/// Quotes an Exec= argument per the Desktop Entry spec.
fn exec_arg(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || "/._-+=:,".contains(c)) {
        s.to_string()
    } else {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('`', "\\`").replace('$', "\\$"))
    }
}

pub fn set_start_with_windows_as(name: &str, exe: &Path, args: &[&str], on: bool) -> std::io::Result<()> {
    let Some(file) = autostart_file(name) else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no config folder"));
    };
    if !on {
        return match std::fs::remove_file(&file) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        };
    }
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut exec = exec_arg(&exe.to_string_lossy());
    for a in args {
        exec.push(' ');
        exec.push_str(&exec_arg(a));
    }
    std::fs::write(
        &file,
        format!("[Desktop Entry]\nType=Application\nName={name}\nExec={exec}\nX-GNOME-Autostart-enabled=true\nNoDisplay=true\n"),
    )
}
