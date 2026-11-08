//! Icons and thumbnails, Linux version. Same worker pool as Windows; thumbnails are made with
//! our own image decoder (`look::decode`).
//!
//! TODO(linux, phase 2): icons from the freedesktop icon theme (MIME type → icon name), and the
//! shared thumbnail cache (~/.cache/thumbnails) so Nautilus / Dolphin thumbnails are reused.
//! Until then `Kind::Icon` fails and the UI shows its built-in glyphs.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;

/// Straight-alpha BGRA, top-down rows.
#[derive(Clone)]
pub struct Bgra {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Icon,
    /// Real thumbnail if there is one; otherwise falls back to the icon.
    Thumbnail,
}

/// Windows-only workaround (GDI+ lifetime); nothing to do here.
pub fn keep_gdiplus_alive() {}

/// Files our decoder can make a thumbnail of.
fn thumbnailable(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "jfif" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff" | "tga" | "qoi" | "exr" | "hdr" | "dds"
            | "pnm" | "pbm" | "pgm" | "ppm" | "svg" | "svgz" | "psd"
    )
}

pub fn get_image(path: &Path, size: u32, kind: Kind) -> anyhow::Result<Bgra> {
    if kind == Kind::Thumbnail && thumbnailable(path) && path.is_file() {
        let d = crate::look::decode::decode_full(path, size.max(16))?;
        if let Some((frame, _)) = d.frames.into_iter().next() {
            return Ok(frame);
        }
    }
    Err(super::unsupported("system icons"))
}

/// Extensions whose icon differs per file (must not be cached by extension).
pub fn has_unique_icon(ext: &str) -> bool {
    // .desktop launchers carry their own Icon=; images show themselves
    matches!(ext, "desktop" | "ico" | "svg" | "png")
}

pub struct Request {
    pub key: u64,
    pub path: PathBuf,
    pub size: u32,
    pub kind: Kind,
}

pub struct Response {
    pub key: u64,
    pub image: Option<Arc<Bgra>>,
    /// Request was discarded (backlog) — not a failure; may be retried.
    pub dropped: bool,
}

/// Background worker pool for icon/thumbnail extraction. Newest requests are served first.
pub struct ThumbPool {
    stack: Arc<Mutex<Vec<Request>>>,
    wake: Sender<()>,
    on_done: Arc<dyn Fn(Response) + Send + Sync>,
}

impl ThumbPool {
    pub fn new(workers: usize, on_done: impl Fn(Response) + Send + Sync + 'static) -> Self {
        let stack: Arc<Mutex<Vec<Request>>> = Arc::new(Mutex::new(Vec::new()));
        let (wake, rx): (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();
        let on_done: Arc<dyn Fn(Response) + Send + Sync> = Arc::new(on_done);
        for i in 0..workers {
            let stack = stack.clone();
            let rx = rx.clone();
            let on_done = on_done.clone();
            std::thread::Builder::new()
                .name(format!("rf-thumb-{i}"))
                .spawn(move || {
                    super::search::lower_thread_priority(10);
                    while rx.recv().is_ok() {
                        loop {
                            let req = stack.lock().pop();
                            let Some(req) = req else { break };
                            let image = get_image(&req.path, req.size, req.kind).ok().map(Arc::new);
                            on_done(Response { key: req.key, image, dropped: false });
                        }
                    }
                })
                .ok();
        }
        Self { stack, wake, on_done }
    }

    pub fn request(&self, req: Request) {
        let mut s = self.stack.lock();
        let dropped: Vec<Request> = if s.len() > 512 { s.drain(0..256).collect() } else { Vec::new() };
        s.push(req);
        drop(s);
        for d in dropped {
            (self.on_done)(Response { key: d.key, image: None, dropped: true });
        }
        let _ = self.wake.send(());
    }

    /// Drops queued work (e.g. on navigation).
    pub fn clear(&self) {
        self.stack.lock().clear();
    }
}
