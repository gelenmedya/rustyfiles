//! Media / document properties (duration, resolution, camera…). TODO(linux, phase 4):
//! FilmCraft container info for video, `lofty` for audio tags, EXIF for photos.

use std::path::Path;

pub fn media_props(_path: &Path) -> Vec<(&'static str, String)> {
    Vec::new()
}
