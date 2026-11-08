//! FTP locations (`ftp://[user[:pass]@]host[:port]/path`). TODO(linux, phase 6): browsing and
//! transfers with `suppaftp`. URL parsing is the same as on Windows.

use std::path::Path;

use rf_core::Entry;

const DEFAULT_PORT: u16 = 21;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtpUrl {
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub pass: Option<String>,
    /// Always starts with '/', no trailing '/' except for the root.
    pub path: String,
}

pub fn is_ftp(p: &Path) -> bool {
    p.to_str().is_some_and(|s| s.len() > 6 && s.as_bytes()[..6].eq_ignore_ascii_case(b"ftp://"))
}

impl FtpUrl {
    pub fn parse(s: &str) -> Option<FtpUrl> {
        let s = s.replace('\\', "/");
        let rest = s.get(..6).filter(|p| p.eq_ignore_ascii_case("ftp://")).map(|_| &s[6..])?;
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        let (cred, hostport) = match authority.rfind('@') {
            Some(i) => (Some(&authority[..i]), &authority[i + 1..]),
            None => (None, authority),
        };
        let (user, pass) = match cred {
            Some(c) => match c.split_once(':') {
                Some((u, p)) => (Some(u.to_string()), Some(p.to_string())),
                None => (Some(c.to_string()), None),
            },
            None => (None, None),
        };
        let (host, port) = match hostport.rsplit_once(':') {
            Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() => (h, p.parse().ok()?),
            _ => (hostport, DEFAULT_PORT),
        };
        if host.is_empty() {
            return None;
        }
        let mut path = path.replace("//", "/");
        while path.len() > 1 && path.ends_with('/') {
            path.pop();
        }
        Some(FtpUrl { host: host.to_string(), port, user, pass, path })
    }
}

fn unsupported() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Unsupported, "FTP is not available on Linux yet")
}

pub fn list(_url: &str) -> std::io::Result<Vec<Entry>> {
    Err(unsupported())
}

pub fn download(_url: &str, _dest: &Path) -> std::io::Result<()> {
    Err(unsupported())
}

pub fn upload(_local: &Path, _dir_url: &str) -> std::io::Result<()> {
    Err(unsupported())
}
