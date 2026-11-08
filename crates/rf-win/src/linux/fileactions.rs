//! Type-specific file actions: set as wallpaper, install a font (for the current user).

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn ext(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

pub fn is_image(p: &Path) -> bool {
    matches!(ext(p).as_str(), "jpg" | "jpeg" | "jfif" | "png" | "bmp" | "gif" | "webp" | "tif" | "tiff" | "heic" | "avif" | "jxl")
}

pub fn is_font(p: &Path) -> bool {
    matches!(ext(p).as_str(), "ttf" | "otf" | "ttc" | "otc" | "woff" | "woff2" | "pfb")
}

/// An executable file (the x bit) that isn't a folder.
pub fn is_runnable(p: &Path) -> bool {
    std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// TODO(linux, phase 5): the xdg-desktop-portal Wallpaper interface.
pub fn set_wallpaper(_path: &Path) -> anyhow::Result<()> {
    Err(super::unsupported("set as wallpaper"))
}

/// Copies the font to `~/.local/share/fonts` and refreshes the font cache.
pub fn install_font_for_user(path: &Path) -> anyhow::Result<PathBuf> {
    anyhow::ensure!(is_font(path), "not a font file");
    let dir = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("no data folder"))?.join("fonts");
    std::fs::create_dir_all(&dir)?;
    let name = path.file_name().ok_or_else(|| anyhow::anyhow!("no file name"))?;
    let dst = dir.join(name);
    anyhow::ensure!(!dst.exists(), "Bu yazı tipi zaten yüklü");
    std::fs::copy(path, &dst)?;
    let _ = std::process::Command::new("fc-cache").arg("-f").arg(&dir).status();
    Ok(dst)
}
