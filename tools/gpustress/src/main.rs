// GPU load generator for the hybrid renderer (D3D11): Windows only. See win.rs.

#[cfg(windows)]
mod win;

#[cfg(windows)]
fn main() {
    win::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("gpustress tests the Windows hybrid renderer (D3D11) and only runs on Windows.");
    std::process::exit(2);
}
