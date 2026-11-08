//! Linux side of the platform layer: the same modules and signatures as the Windows code next
//! to it (see `lib.rs`), so `rf-app` compiles unchanged.
//!
//! Port status (PORT_PLAN.md, phase 1): listing, search, folder sizes, watch, known folders,
//! drives, single instance, open / open-with and image thumbnails work. Everything marked
//! `TODO(linux)` is a stub: it returns [`unsupported`] or an empty result, never panics.

pub mod accent;
pub mod clipboard;
pub mod ctxmenu;
pub mod default_fm;
pub mod dialog_reg;
pub mod dialogs;
pub mod dnd;
pub mod drives;
pub mod encoding;
pub mod fastcopy;
pub mod fastdelete;
pub mod fileactions;
pub mod fileinfo;
pub mod fileops;
pub mod foldersize;
pub mod ftp;
pub mod jumplist;
pub mod keys;
pub mod known;
pub mod libraries;
pub mod listing;
pub mod localized;
pub mod look;
pub mod network;
pub mod preview_host;
pub mod props;
pub mod recycle;
pub mod removal;
pub mod rotate;
pub mod search;
pub mod shell;
pub mod shortcut;
pub mod single_instance;
pub mod sta;
pub mod sysparams;
pub mod thumbs;
pub mod tray;
pub mod volume;
pub mod window;
pub mod wsearch;

/// The error every not-yet-ported function returns.
pub(crate) fn unsupported(what: &str) -> anyhow::Error {
    anyhow::anyhow!("{what}: not available on Linux yet")
}

/// Unix time in milliseconds (0 when unknown or before 1970).
pub(crate) fn unix_ms(t: std::io::Result<std::time::SystemTime>) -> i64 {
    t.ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_millis() as i64)
}
