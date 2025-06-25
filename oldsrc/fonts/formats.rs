//! Font format support

/// Supported font formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFormat {
    TrueType,
    OpenType,
    PostScript,
    Bitmap,
}

impl FontFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ttf" => Some(FontFormat::TrueType),
            "otf" => Some(FontFormat::OpenType),
            "ps" | "pfa" | "pfb" => Some(FontFormat::PostScript),
            "bdf" | "pcf" => Some(FontFormat::Bitmap),
            _ => None,
        }
    }
}
