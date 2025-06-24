//! Font cache implementation

use crate::types::Result;
use std::collections::HashMap;

/// Font cache for managing loaded fonts
#[derive(Debug)]
pub struct FontCache {
    cache: HashMap<String, CachedFont>,
    max_size: usize,
}

#[derive(Debug, Clone)]
struct CachedFont {
    data: Vec<u8>,
    size: usize,
}

impl FontCache {
    /// Create new font cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 1024 * 1024 * 100, // 100MB
        }
    }

    /// Initialize the cache
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing font cache");
        Ok(())
    }

    /// Cache a font
    pub fn cache_font(&mut self, name: String, data: Vec<u8>) -> Result<()> {
        let size = data.len();
        let cached_font = CachedFont { data, size };
        self.cache.insert(name, cached_font);
        Ok(())
    }

    /// Get cached font
    pub fn get_font(&self, name: &str) -> Option<&[u8]> {
        self.cache.get(name).map(|font| font.data.as_slice())
    }
}
