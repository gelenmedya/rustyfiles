//! Facts for the Properties window: times, attributes, size on disk.
//! TODO(linux, phase 5): owner / group / permission bits, and the default app's name.

use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct Info {
    pub is_dir: bool,
    pub size: u64,
    /// Allocated size (sparse files, block rounding). Files only.
    pub size_on_disk: Option<u64>,
    pub created_ms: i64,
    pub modified_ms: i64,
    pub accessed_ms: i64,
    /// Windows-style attribute bits (see `rf_core::attrs`).
    pub attrs: u32,
    /// Friendly name of the app that opens this type. Files only.
    pub opens_with: Option<String>,
}

pub fn info(path: &Path) -> std::io::Result<Info> {
    let link = std::fs::symlink_metadata(path)?.file_type().is_symlink();
    let m = std::fs::metadata(path)?;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(Info {
        is_dir: m.is_dir(),
        size: if m.is_dir() { 0 } else { m.len() },
        size_on_disk: (!m.is_dir()).then(|| m.blocks() * 512),
        created_ms: super::unix_ms(m.created()),
        modified_ms: super::unix_ms(m.modified()),
        accessed_ms: super::unix_ms(m.accessed()),
        attrs: super::listing::attrs_of(&name, link, Some(&m), m.is_dir()),
        opens_with: None,
    })
}

pub fn size_on_disk(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().filter(|m| !m.is_dir()).map(|m| m.blocks() * 512)
}

pub fn opens_with(_path: &Path) -> Option<String> {
    None
}

/// Read-only = no write bits; "hidden" is a leading dot on Linux and can't be toggled here.
pub fn set_flags(path: &Path, readonly: Option<bool>, _hidden: Option<bool>) -> std::io::Result<()> {
    if let Some(ro) = readonly {
        let mut p = std::fs::metadata(path)?.permissions();
        let mode = p.mode();
        // read-only: drop every write bit; writable again: give the owner write access back
        p.set_mode(if ro { mode & !0o222 } else { mode | 0o200 });
        std::fs::set_permissions(path, p)?;
    }
    Ok(())
}
