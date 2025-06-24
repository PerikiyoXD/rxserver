//! Font manager implementation

use super::types::{FontFamily, FontInfo, FontStyle, FontWeight};
use crate::types::Result;
use std::collections::HashMap;

/// Font manager for handling font operations
#[derive(Debug, Clone)]
pub struct FontManager {
    fonts: HashMap<String, FontInfo>,
    font_paths: Vec<std::path::PathBuf>,
}

impl FontManager {
    /// Create new font manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            fonts: HashMap::new(),
            font_paths: Vec::new(),
        })
    }

    /// Scan system fonts
    pub async fn scan_system_fonts(&mut self) -> Result<()> {
        tracing::info!("Scanning system fonts");

        // Add common font directories
        self.font_paths.push("/usr/share/fonts".into());
        self.font_paths.push("/System/Library/Fonts".into());
        self.font_paths.push("C:\\Windows\\Fonts".into());

        // TODO: Actually scan directories for fonts

        tracing::info!("Found {} fonts", self.fonts.len());
        Ok(())
    }

    /// Get loaded fonts
    pub fn fonts(&self) -> &HashMap<String, FontInfo> {
        &self.fonts
    }
}
