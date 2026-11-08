//! rf-win: the platform layer (fast enumeration, shell integration, icons/thumbnails, file ops,
//! watcher). Windows is the reference implementation; `src/linux/` mirrors the same API with
//! Linux code (or, while the port is in progress, with stubs that report "unsupported").
//!
//! Every function here may block (disk, network, shell extensions). Never call them on the UI thread.

// ---------------------------------------------------------------------- every platform
pub mod compat;
pub mod complete;
pub mod git;
pub mod hash;
pub mod input;
pub mod process;
pub mod watch;

// ---------------------------------------------------------------------- Windows
#[cfg(windows)]
pub mod accent;
#[cfg(windows)]
pub mod clipboard;
#[cfg(windows)]
pub mod ctxmenu;
#[cfg(windows)]
pub mod default_fm;
#[cfg(windows)]
pub mod dialog_reg;
#[cfg(windows)]
pub mod dialogs;
#[cfg(windows)]
pub mod dnd;
#[cfg(windows)]
pub mod drives;
#[cfg(windows)]
pub mod encoding;
#[cfg(windows)]
pub mod fastcopy;
#[cfg(windows)]
pub mod fastdelete;
#[cfg(windows)]
pub mod fileops;
#[cfg(windows)]
pub mod foldersize;
#[cfg(windows)]
pub mod sysparams;
#[cfg(windows)]
pub mod known;
#[cfg(windows)]
pub mod libraries;
#[cfg(windows)]
pub mod ftp;
#[cfg(windows)]
pub mod keys;
#[cfg(windows)]
pub mod look;
#[cfg(windows)]
pub mod listing;
#[cfg(windows)]
pub mod localized;
#[cfg(windows)]
pub mod network;
#[cfg(windows)]
pub mod fileactions;
#[cfg(windows)]
pub mod fileinfo;
#[cfg(windows)]
pub mod jumplist;
#[cfg(windows)]
pub mod wsearch;
#[cfg(windows)]
pub mod preview_host;
#[cfg(windows)]
pub mod props;
#[cfg(windows)]
pub mod recycle;
#[cfg(windows)]
pub mod search;
#[cfg(windows)]
pub mod shell;
#[cfg(windows)]
pub mod shortcut;
#[cfg(windows)]
pub mod single_instance;
#[cfg(windows)]
pub mod sta;
#[cfg(windows)]
pub mod removal;
#[cfg(windows)]
pub mod rotate;
#[cfg(windows)]
pub mod thumbs;
#[cfg(windows)]
pub mod tray;
#[cfg(windows)]
pub mod window;
#[cfg(windows)]
pub mod volume;
#[cfg(windows)]
pub mod wide;

// ---------------------------------------------------------------------- Linux (and other Unix)
#[cfg(not(windows))]
mod linux;
#[cfg(not(windows))]
pub use linux::*;

pub use drives::{Drive, DriveKind, list_drives};
pub use known::{KnownFolder, known_folders};
pub use listing::{ListError, list_dir};
