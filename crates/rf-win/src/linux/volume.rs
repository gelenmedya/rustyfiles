//! Volume details (Properties of a drive). TODO(linux, phase 6): label and serial from UDisks2,
//! eject through UDisks2.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeInfo {
    pub root: PathBuf,
    pub label: String,
    pub filesystem: String,
    pub serial: u32,
    pub total: u64,
    pub free: u64,
    /// Turkish, e.g. "Yerel Disk".
    pub drive_type: String,
}

impl VolumeInfo {
    pub fn serial_string(&self) -> String {
        format!("{:04X}-{:04X}", self.serial >> 16, self.serial & 0xFFFF)
    }
}

pub fn drive_type_name(t: u32) -> &'static str {
    match t {
        2 => "Çıkarılabilir Disk",
        3 => "Yerel Disk",
        4 => "Ağ Sürücüsü",
        5 => "CD/DVD Sürücüsü",
        6 => "RAM Disk",
        _ => "Bilinmeyen",
    }
}

/// Details of the mount holding `root`.
pub fn volume_info(root: &Path) -> anyhow::Result<VolumeInfo> {
    use super::drives::DriveKind;
    let d = super::drives::list_drives()
        .into_iter()
        .filter(|d| root.starts_with(&d.root))
        .max_by_key(|d| d.root.as_os_str().len())
        .ok_or_else(|| anyhow::anyhow!("Sürücü bulunamadı: {}", root.display()))?;
    let t = match d.kind {
        DriveKind::Removable => 2,
        DriveKind::Fixed => 3,
        DriveKind::Network => 4,
        DriveKind::Optical => 5,
        DriveKind::RamDisk => 6,
        DriveKind::Unknown => 0,
    };
    Ok(VolumeInfo {
        root: d.root,
        label: d.label,
        filesystem: d.fs,
        serial: 0,
        total: d.total,
        free: d.free,
        drive_type: drive_type_name(t).to_string(),
    })
}

pub fn eject(_drive_letter: char) -> anyhow::Result<()> {
    Err(super::unsupported("eject"))
}
