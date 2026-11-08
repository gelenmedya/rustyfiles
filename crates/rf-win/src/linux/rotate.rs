//! Rotating images 90°. TODO(linux, phase 4): rewrite the EXIF orientation tag for JPEG and
//! rotate the pixels (image crate) for PNG/BMP/TIFF/GIF, via a temp file + rename.

use std::path::Path;

/// Formats we will be able to rotate. Empty until the port lands, so the menu item stays hidden.
pub fn can_rotate(_p: &Path) -> bool {
    false
}

/// Next EXIF orientation after a 90° turn (mirrored variants included).
pub fn next_orientation(cur: u16, clockwise: bool) -> u16 {
    // 1 → 6 → 3 → 8 → 1 (plain); 2 → 7 → 4 → 5 → 2 (mirrored)
    const CW: [(u16, u16); 8] = [(1, 6), (6, 3), (3, 8), (8, 1), (2, 7), (7, 4), (4, 5), (5, 2)];
    let cur = if (1..=8).contains(&cur) { cur } else { 1 };
    if clockwise {
        CW.iter().find(|(a, _)| *a == cur).map_or(6, |(_, b)| *b)
    } else {
        CW.iter().find(|(_, b)| *b == cur).map_or(8, |(a, _)| *a)
    }
}

pub fn rotate(_path: &Path, _clockwise: bool) -> anyhow::Result<()> {
    Err(super::unsupported("rotating images"))
}
