//! "Set as default file manager". TODO(linux, phase 5): `xdg-mime default rustyfiles.desktop
//! inode/directory` plus the `org.freedesktop.FileManager1` D-Bus interface. The registry
//! arguments (`classes_root`) are kept so the call sites stay the same.

pub const CLASSES: &str = r"Software\Classes";
pub const ARCHIVE_EXTS: [&str; 9] = [".zip", ".7z", ".rar", ".tar", ".tgz", ".txz", ".tzst", ".tbz2", ".tbz"];
pub const APP_NAME: &str = "rustyfiles";

pub fn enable(_exe: &str, _classes_root: &str) -> anyhow::Result<()> {
    Err(super::unsupported("default file manager"))
}

pub fn register_app(_exe: &str, _classes_root: &str) -> anyhow::Result<()> {
    Ok(())
}

pub fn add_archives(_exe: &str, _classes_root: &str) -> bool {
    false
}

pub fn disable(_classes_root: &str) -> anyhow::Result<()> {
    Ok(())
}

pub fn is_enabled(_classes_root: &str) -> bool {
    false
}

pub fn registered_exe(_classes_root: &str) -> Option<std::path::PathBuf> {
    None
}

pub fn heal(_current: &str, _classes_root: &str) -> bool {
    false
}

/// Windows shell namespaces we hand back to Explorer. There is no Explorer here.
pub fn should_forward_to_explorer(_arg: &str) -> bool {
    false
}

/// Always "yes", so nothing is ever handed to Explorer.
pub fn forwarded_recently(_arg: &str) -> bool {
    true
}

pub fn note_forward(_arg: &str) {}
