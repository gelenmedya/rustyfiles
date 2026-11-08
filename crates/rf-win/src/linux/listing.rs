//! Directory listing with `std::fs::read_dir` (getdents64). Same contract as the Windows
//! version: entries stream to `on_batch` in chunks so the first items show at once.

use std::fs::{DirEntry, Metadata};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use rf_core::{Entry, EntryKind, attrs};

#[derive(Debug, thiserror::Error)]
pub enum ListError {
    #[error("Klasör bulunamadı")]
    NotFound,
    #[error("Erişim reddedildi")]
    AccessDenied,
    #[error("Geçersiz klasör adı")]
    InvalidName,
    #[error("Sürücü hazır değil")]
    NotReady,
    #[error("Ağ yolu bulunamadı")]
    NetworkPath,
    #[error("Listeleme hatası (kod {0})")]
    Os(u32),
    #[error("İptal edildi")]
    Cancelled,
}

impl From<std::io::Error> for ListError {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind::*;
        match e.kind() {
            NotFound => ListError::NotFound,
            PermissionDenied => ListError::AccessDenied,
            _ => match e.raw_os_error() {
                // ENOTDIR / ENAMETOOLONG / EINVAL
                Some(20) | Some(36) | Some(22) => ListError::InvalidName,
                // ENOMEDIUM / ENXIO: empty card reader, ejected disc
                Some(123) | Some(6) => ListError::NotReady,
                // EHOSTDOWN / EHOSTUNREACH / ENETUNREACH / ESTALE (NFS)
                Some(112) | Some(113) | Some(101) | Some(116) => ListError::NetworkPath,
                Some(code) => ListError::Os(code as u32),
                None => ListError::Os(0),
            },
        }
    }
}

/// Lists `dir`, streaming entries to `on_batch` in chunks of `batch_size`. `cancelled` is
/// polled between batches.
pub fn list_dir(
    dir: &Path,
    batch_size: usize,
    cancelled: &dyn Fn() -> bool,
    on_batch: &mut dyn FnMut(Vec<Entry>),
) -> Result<usize, ListError> {
    let rd = std::fs::read_dir(dir)?;
    let mut total = 0usize;
    let mut batch = Vec::with_capacity(batch_size.min(4096));
    for de in rd {
        let Ok(de) = de else { continue };
        let Some(e) = to_entry(&de) else { continue };
        batch.push(e);
        if batch.len() >= batch_size {
            total += batch.len();
            on_batch(std::mem::replace(&mut batch, Vec::with_capacity(batch_size.min(4096))));
            if cancelled() {
                return Err(ListError::Cancelled);
            }
        }
    }
    if !batch.is_empty() {
        total += batch.len();
        on_batch(batch);
    }
    Ok(total)
}

/// Windows-style attribute bits for a Unix file, so the shared UI code works unchanged:
/// dot files are HIDDEN, symlinks are REPARSE_POINT, no write bit is READONLY.
pub(crate) fn attrs_of(name: &str, link: bool, meta: Option<&Metadata>, is_dir: bool) -> u32 {
    let mut a = 0;
    if name.starts_with('.') {
        a |= attrs::HIDDEN;
    }
    if link {
        a |= attrs::REPARSE_POINT;
    }
    if is_dir {
        a |= attrs::DIRECTORY;
    }
    if meta.is_some_and(|m| m.permissions().mode() & 0o222 == 0) {
        a |= attrs::READONLY;
    }
    a
}

/// One entry for a path outside a listing (tag views). `follow_links`: describe a link's
/// target instead of the link.
pub fn entry_for(path: &Path, follow_links: bool) -> Option<Entry> {
    let link = std::fs::symlink_metadata(path).ok()?.file_type().is_symlink();
    let m = if follow_links { std::fs::metadata(path) } else { std::fs::symlink_metadata(path) }.ok()?;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let is_dir = m.is_dir();
    let kind = if is_dir { EntryKind::Dir } else { EntryKind::File };
    Some(Entry::new(
        &name,
        kind,
        if is_dir { 0 } else { m.len() },
        super::unix_ms(m.modified()),
        super::unix_ms(m.created()),
        attrs_of(&name, link, Some(&m), is_dir),
    ))
}

/// One listing entry. Symlinks show as what they point to (a link to a folder is a folder);
/// a broken link is a file.
pub(crate) fn to_entry(de: &DirEntry) -> Option<Entry> {
    let name = de.file_name();
    let name = name.to_string_lossy();
    let link = de.file_type().ok()?.is_symlink();
    let meta = if link { std::fs::metadata(de.path()).or_else(|_| de.metadata()) } else { de.metadata() }.ok();
    let is_dir = meta.as_ref().is_some_and(|m| m.is_dir());
    let kind = if is_dir { EntryKind::Dir } else { EntryKind::File };
    let size = if is_dir { 0 } else { meta.as_ref().map_or(0, |m| m.len()) };
    let modified = meta.as_ref().map_or(0, |m| super::unix_ms(m.modified()));
    let created = meta.as_ref().map_or(0, |m| super::unix_ms(m.created()));
    Some(Entry::new(&name, kind, size, modified, created, attrs_of(&name, link, meta.as_ref(), is_dir)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_temp_dir() {
        let dir = std::env::temp_dir().join(format!("rf-list-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        for i in 0..250 {
            std::fs::write(dir.join(format!("f{i}.txt")), b"hello").unwrap();
        }
        std::fs::write(dir.join(".hidden"), b"").unwrap();
        let mut all = Vec::new();
        let n = list_dir(&dir, 100, &|| false, &mut |b| all.extend(b)).unwrap();
        assert_eq!(n, 252);
        assert!(all.iter().any(|e| e.is_dir() && &*e.name == "sub"));
        assert!(all.iter().any(|e| &*e.name == "f7.txt" && e.size == 5 && &*e.ext == "txt"));
        assert!(all.iter().any(|e| &*e.name == ".hidden" && e.is_hidden()));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_dir() {
        let r = list_dir(Path::new("/definitely/not/here/rf"), 100, &|| false, &mut |_| {});
        assert!(matches!(r, Err(ListError::NotFound)));
    }
}
