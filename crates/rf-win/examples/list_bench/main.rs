// Uses Win32 directly: Windows only. See win.rs.

#[cfg(windows)]
mod win;

#[cfg(windows)]
fn main() {
    win::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("list_bench is a Windows-only example.");
}
