//! Single-instance plumbing, Linux version: the first process holds an `flock` on a lock file
//! and serves a Unix socket; later launches send their arguments (one line) and exit.
//! Both live in `$XDG_RUNTIME_DIR` (per user, cleared at logout).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

/// (lock file, socket) for an app: rustyfiles and DirectLook are separate instance groups.
fn names_for(app: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
    // A separate settings folder (tests, portable copies) is a separate instance group.
    let suffix = match std::env::var_os("RUSTYFILES_CONFIG_DIR") {
        Some(d) => {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            d.hash(&mut h);
            format!("-{:08x}", h.finish() as u32)
        }
        None => String::new(),
    };
    (dir.join(format!("{app}-{user}{suffix}.lock")), dir.join(format!("{app}-{user}{suffix}.sock")))
}

pub enum Role {
    /// We are the first instance. Keep the guard alive for the process lifetime.
    Primary(InstanceGuard),
    /// Another instance is running and received our message.
    Forwarded,
}

/// Holds the lock file (and its `flock`) while alive.
pub struct InstanceGuard(#[allow(dead_code)] Option<std::fs::File>);

/// Decides the role. `message` is sent to the primary when one exists ("" = just focus a new window/tab).
pub fn acquire(message: &str) -> Role {
    acquire_app("rustyfiles", message)
}

/// [`acquire`] for another application of the bundle (e.g. "directlook").
pub fn acquire_app(app: &str, message: &str) -> Role {
    let (lock, sock) = names_for(app);
    let Ok(f) = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(&lock) else {
        return Role::Primary(InstanceGuard(None));
    };
    if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
        return Role::Primary(InstanceGuard(Some(f)));
    }
    // Primary may still be starting its socket: retry briefly.
    for _ in 0..40 {
        if send(&sock, message) {
            return Role::Forwarded;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    // Primary unresponsive (hung or exiting): run on our own.
    Role::Primary(InstanceGuard(None))
}

/// Sends `message` to the running instance of `app` (false if none answers).
pub fn send_to(app: &str, message: &str) -> bool {
    send(&names_for(app).1, message)
}

fn send(sock: &std::path::Path, message: &str) -> bool {
    let Ok(mut s) = UnixStream::connect(sock) else { return false };
    let _ = s.set_write_timeout(Some(Duration::from_secs(2)));
    let mut bytes = message.as_bytes().to_vec();
    bytes.push(b'\n');
    s.write_all(&bytes).is_ok()
}

/// Serves the socket on a background thread; each received line is passed to `on_message`.
pub fn serve(on_message: impl Fn(String) + Send + Sync + 'static) {
    serve_app("rustyfiles", on_message)
}

/// [`serve`] for another application of the bundle.
pub fn serve_app(app: &str, on_message: impl Fn(String) + Send + Sync + 'static) {
    let on_message = std::sync::Arc::new(on_message);
    let (_, sock) = names_for(app);
    // a socket file left by a crashed instance (we hold the lock, so nobody else serves it)
    let _ = std::fs::remove_file(&sock);
    let listener = match UnixListener::bind(&sock) {
        Ok(l) => l,
        Err(e) => {
            log::warn!("single instance socket {}: {e}", sock.display());
            return;
        }
    };
    let _ = std::thread::Builder::new().name("rf-instance-sock".into()).spawn(move || {
        for conn in listener.incoming() {
            let Ok(conn) = conn else { continue };
            let cb = on_message.clone();
            // each client on its own thread so a stalled client can't block the next one
            std::thread::spawn(move || {
                let _ = conn.set_read_timeout(Some(Duration::from_secs(5)));
                for line in BufReader::new(conn).lines() {
                    match line {
                        Ok(l) => cb(l),
                        Err(_) => break,
                    }
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_instance_forwards() {
        let app = format!("rf-test-{}", std::process::id());
        let Role::Primary(_guard) = acquire_app(&app, "") else { panic!("first must be primary") };
        let (tx, rx) = std::sync::mpsc::channel();
        serve_app(&app, move |m| {
            let _ = tx.send(m);
        });
        assert!(send_to(&app, "/tmp/x"));
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap(), "/tmp/x");
    }
}
