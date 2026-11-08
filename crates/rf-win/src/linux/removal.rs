//! "Safely remove": Windows asks every process before ejecting a volume. On Linux, UDisks2
//! unmounts (and fails while files are open). TODO(linux, phase 6): watch UDisks2 so the app can
//! let go of a drive before it is unmounted.

use std::sync::mpsc::Sender;

pub fn start(_on_query_remove: impl Fn(char, Sender<()>) + Send + 'static) {}

pub fn registered() -> Vec<char> {
    Vec::new()
}
