//! Tray icon and notifications. TODO(linux, phase 5): a StatusNotifierItem tray icon (`ksni`).
//! Notifications already work through `notify-send` (libnotify).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    /// Click on the icon (or "Open" in its menu).
    Open,
    NewWindow,
    Quit,
}

#[derive(Clone)]
pub struct Labels {
    pub tooltip: String,
    pub open: String,
    pub new_window: String,
    pub quit: String,
}

pub fn set_handler(_f: impl Fn(TrayEvent) + Send + 'static) {}

pub fn show(_labels: Labels) {}

pub fn hide() {}

pub fn notify(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["--app-name=rustyfiles", title, body])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|mut c| std::thread::spawn(move || c.wait()));
}
