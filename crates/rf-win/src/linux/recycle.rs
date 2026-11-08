//! The trash. TODO(linux, phase 3): the freedesktop Trash spec (`trash` crate): list, restore,
//! empty, per-volume `.Trash-$UID`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecycleItem {
    /// Original file name.
    pub name: String,
    /// Folder the item was deleted from.
    pub original_location: String,
    /// Deletion time, Unix milliseconds (UTC); 0 when unknown.
    pub deleted_ms: i64,
    pub size: u64,
    /// Real location inside the trash; also the key used by [`restore`].
    pub parsing_path: String,
    pub is_dir: bool,
}

pub fn list() -> anyhow::Result<Vec<RecycleItem>> {
    Ok(Vec::new())
}

pub fn restore(_items: &[RecycleItem]) -> anyhow::Result<()> {
    Err(super::unsupported("restore from trash"))
}

pub fn empty(_confirm: bool) -> anyhow::Result<()> {
    Err(super::unsupported("empty trash"))
}

/// (item count, total bytes).
pub fn info() -> anyhow::Result<(u64, u64)> {
    Ok((0, 0))
}
