// Test window for the D3D11 presenter: Windows only. See win.rs.

#[cfg(windows)]
mod win;

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    win::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("play_window tests the D3D11 presenter and only runs on Windows.");
}
