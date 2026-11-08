//! The user's standard folders, from xdg-user-dirs (`~/.config/user-dirs.dirs`) via `dirs`.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnownFolder {
    Desktop,
    Downloads,
    Documents,
    Pictures,
    Music,
    Videos,
    Profile,
}

impl KnownFolder {
    pub const PINNED_DEFAULTS: [KnownFolder; 6] = [
        KnownFolder::Desktop,
        KnownFolder::Downloads,
        KnownFolder::Documents,
        KnownFolder::Pictures,
        KnownFolder::Music,
        KnownFolder::Videos,
    ];

    pub fn title(self) -> &'static str {
        match self {
            KnownFolder::Desktop => "Masaüstü",
            KnownFolder::Downloads => "İndirilenler",
            KnownFolder::Documents => "Belgeler",
            KnownFolder::Pictures => "Resimler",
            KnownFolder::Music => "Müzik",
            KnownFolder::Videos => "Videolar",
            KnownFolder::Profile => "Kullanıcı",
        }
    }

    pub fn path(self) -> Option<PathBuf> {
        let p = match self {
            KnownFolder::Desktop => dirs::desktop_dir(),
            KnownFolder::Downloads => dirs::download_dir(),
            KnownFolder::Documents => dirs::document_dir(),
            KnownFolder::Pictures => dirs::picture_dir(),
            KnownFolder::Music => dirs::audio_dir(),
            KnownFolder::Videos => dirs::video_dir(),
            KnownFolder::Profile => dirs::home_dir(),
        }?;
        // xdg-user-dirs points unset folders at $HOME: don't pin the home folder six times
        if self != KnownFolder::Profile && dirs::home_dir().as_ref() == Some(&p) {
            return None;
        }
        Some(p)
    }
}

pub fn known_folders() -> Vec<(KnownFolder, PathBuf)> {
    KnownFolder::PINNED_DEFAULTS
        .iter()
        .filter_map(|k| k.path().filter(|p| p.is_dir()).map(|p| (*k, p)))
        .collect()
}
