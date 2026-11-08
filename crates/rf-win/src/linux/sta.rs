//! Windows needs one long-lived COM "shell thread" for menus and dialogs. Linux doesn't, but
//! the callers still expect the same behaviour: jobs run off the UI thread, one at a time, in
//! order. So this is a single worker thread with a queue.

use std::sync::OnceLock;
use std::sync::mpsc::{Sender, channel};

use crate::compat::HWND;

type Job = Box<dyn FnOnce(HWND) + Send>;

fn queue() -> &'static std::sync::Mutex<Sender<Job>> {
    static Q: OnceLock<std::sync::Mutex<Sender<Job>>> = OnceLock::new();
    Q.get_or_init(|| {
        let (tx, rx) = channel::<Job>();
        let _ = std::thread::Builder::new().name("rf-shell".into()).spawn(move || {
            for job in rx {
                job(HWND::default());
            }
        });
        std::sync::Mutex::new(tx)
    })
}

/// Runs `f` on the shell thread (the owner handle is a placeholder here).
pub fn run(f: impl FnOnce(HWND) + Send + 'static) {
    let _ = queue().lock().unwrap().send(Box::new(f));
}
