//! Windows Libraries (virtual merged folders). Linux has no such concept: none are listed.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub file: std::path::PathBuf,
    pub name: String,
    pub folders: Vec<std::path::PathBuf>,
    pub default_save: Option<std::path::PathBuf>,
}

pub fn load(_file: &std::path::Path) -> Option<Library> {
    None
}

pub fn list() -> Vec<Library> {
    Vec::new()
}
