//! Shell actions (open, open with, terminal) through the freedesktop tools.
//!
//! TODO(linux, phase 2): parse .desktop files / mimeapps.list ourselves (Open with list,
//! default-app names) instead of handing everything to `xdg-open`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// COM does not exist here; kept so shared call sites compile.
pub fn ensure_com() {}

/// Starts a detached helper; its exit status is not waited for (the app may take a while).
fn spawn(cmd: &mut Command) -> anyhow::Result<()> {
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = cmd.spawn()?;
    // reap it in the background so it doesn't linger as a zombie
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Opens `path` with its default application (folders: the default file manager).
pub fn open(path: &Path) -> anyhow::Result<()> {
    spawn(Command::new("xdg-open").arg(path))
}

/// "Open with…". TODO(linux): our own app chooser; for now the portal's (GNOME/KDE show one).
pub fn open_with(path: &Path) -> anyhow::Result<()> {
    // `gio open --ask` does not exist; the OpenURI portal asks when `ask` is set.
    let uri = format!("file://{}", path.display());
    let status = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.freedesktop.portal.Desktop",
            "--object-path",
            "/org/freedesktop/portal/desktop",
            "--method",
            "org.freedesktop.portal.OpenURI.OpenURI",
            "",
            &uri,
            "{'ask': <true>}",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => open(path),
    }
}

/// The system Properties dialog. Linux has none: the app shows its own properties window.
pub fn properties(_path: &Path) -> anyhow::Result<()> {
    Err(super::unsupported("system properties dialog"))
}

/// Opens a terminal in `dir`: `xdg-terminal-exec`, then `$TERMINAL`, then common terminals.
pub fn open_terminal(dir: &Path) -> anyhow::Result<()> {
    let mut candidates: Vec<String> = vec!["xdg-terminal-exec".into()];
    if let Ok(t) = std::env::var("TERMINAL") {
        candidates.push(t);
    }
    candidates.extend(
        ["kgx", "gnome-terminal", "konsole", "xfce4-terminal", "alacritty", "kitty", "foot", "wezterm", "xterm"].map(String::from),
    );
    for c in candidates {
        if spawn(Command::new(&c).current_dir(dir)).is_ok() {
            return Ok(());
        }
    }
    anyhow::bail!("no terminal emulator found (set $TERMINAL)")
}

/// A blocking yes/no box. TODO(linux): the app should ask in its own UI; until then this
/// tries `zenity`/`kdialog` and otherwise answers no (the safe choice).
pub fn confirm(_owner: crate::compat::HWND, title: &str, text: &str) -> bool {
    for (exe, args) in [
        ("zenity", vec!["--question".to_string(), format!("--title={title}"), format!("--text={text}")]),
        ("kdialog", vec!["--title".to_string(), title.to_string(), "--yesno".to_string(), text.to_string()]),
    ] {
        if let Ok(s) = Command::new(exe).args(&args).stdin(Stdio::null()).status() {
            return s.success();
        }
    }
    log::warn!("confirm without a dialog tool, answering no: {title}: {text}");
    false
}

/// A blocking message box (see [`confirm`]).
pub fn alert(_owner: crate::compat::HWND, title: &str, text: &str) {
    for (exe, args) in [
        ("zenity", vec!["--info".to_string(), format!("--title={title}"), format!("--text={text}")]),
        ("kdialog", vec!["--title".to_string(), title.to_string(), "--msgbox".to_string(), text.to_string()]),
    ] {
        if Command::new(exe).args(&args).stdin(Stdio::null()).status().is_ok() {
            return;
        }
    }
    log::warn!("{title}: {text}");
}

/// `shell:` names have no meaning here; `~` and `~/…` are the closest thing.
pub fn resolve_shell_path(s: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    if s == "~" {
        return Some(home);
    }
    s.strip_prefix("~/").map(|rest| home.join(rest))
}

/// Windows-only (critical-error popups for empty drives).
pub fn suppress_critical_error_dialogs() {}

/// Windows tells Explorer windows to refresh; inotify does that here.
pub fn notify_updated(_dir: &Path) {}
