//! "rustyfiles in other apps' Open / Save dialogs" — a Windows COM registration. The Linux
//! equivalent would be an xdg-desktop-portal FileChooser backend: out of scope for now.

pub const CLASSES: &str = r"Software\Classes";
pub const DLL_NAME: &str = "rustyfiles_dialog.dll";

pub fn enable(_dll: &str, _classes_root: &str) -> anyhow::Result<()> {
    Err(super::unsupported("replacing other apps' file dialogs"))
}

pub fn disable(_classes_root: &str) -> anyhow::Result<()> {
    Ok(())
}

pub fn is_enabled(_classes_root: &str) -> bool {
    false
}

pub fn point_to(_dll: &str, _classes_root: &str) -> bool {
    false
}

pub fn heal(_dll: &str, _classes_root: &str) -> bool {
    false
}
