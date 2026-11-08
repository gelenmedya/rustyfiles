//! PDF pages for DirectLook. Windows uses its built-in PDF engine. TODO(linux, phase 4):
//! pdfium (`pdfium-render`) or poppler. Until then `open` fails and DirectLook shows file info.

use std::path::Path;

use crate::thumbs::Bgra;

pub const MAX_WIDTH: u32 = 4096;

pub struct Pdf(());

#[derive(Debug, PartialEq, Eq)]
pub enum PdfError {
    /// Protected with a password.
    Password,
    /// Not readable as a PDF (damaged, or not a PDF).
    Unreadable,
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PdfError::Password => "PDF parola korumalı",
            PdfError::Unreadable => "PDF okunamadı",
        })
    }
}

impl std::error::Error for PdfError {}

impl Pdf {
    pub fn open(_path: &Path, _password: Option<&str>) -> anyhow::Result<Pdf> {
        Err(crate::linux::unsupported("PDF preview"))
    }

    pub fn page_count(&self) -> u32 {
        0
    }

    pub fn page_size(&self, _index: u32) -> anyhow::Result<(f32, f32)> {
        Err(crate::linux::unsupported("PDF preview"))
    }

    pub fn render(&self, _index: u32, _width: u32) -> anyhow::Result<Bgra> {
        Err(crate::linux::unsupported("PDF preview"))
    }
}
