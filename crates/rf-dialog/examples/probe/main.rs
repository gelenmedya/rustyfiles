// Uses Win32 directly: Windows only. See win.rs.

#[cfg(windows)]
mod win;

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    win::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("probe is a Windows-only example.");
}
