//! Font management system
//!
//! This module provides font loading, caching, and rendering capabilities
//! for the X11 server.

pub mod cache;
pub mod formats;
pub mod manager;
pub mod registry;
pub mod rendering;
pub mod types;

// Re-export commonly used types
pub use cache::FontCache;
pub use manager::FontManager;
pub use registry::{FontError, FontInfo as RegistryFontInfo, FontRegistry, FontResource};
pub use rendering::FontRenderer;
pub use types::{FontFamily, FontInfo, FontStyle, FontWeight};

use crate::types::Result;
use std::sync::Arc;

/// Font system coordinator
#[derive(Debug)]
pub struct FontSystem {
    manager: FontManager,
    cache: FontCache,
    renderer: FontRenderer,
    registry: FontRegistry,
}

impl FontSystem {
    /// Create a new font system
    pub fn new() -> Result<Self> {
        let mut registry = FontRegistry::new();
        let manager = FontManager::new()?;

        // Set up the registry with the manager
        registry.set_font_manager(Arc::new(manager.clone()));

        Ok(Self {
            manager,
            cache: FontCache::new(),
            renderer: FontRenderer::new()?,
            registry,
        })
    }

    /// Initialize the font system
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing font system");

        self.manager.scan_system_fonts().await?;
        self.cache.initialize().await?;
        self.renderer.initialize().await?;

        tracing::info!("Font system initialized successfully");
        Ok(())
    }

    /// Get the font manager
    pub fn manager(&self) -> &FontManager {
        &self.manager
    }

    /// Get the font cache
    pub fn cache(&self) -> &FontCache {
        &self.cache
    }

    /// Get the font renderer
    pub fn renderer(&self) -> &FontRenderer {
        &self.renderer
    }

    /// Get the font registry
    pub fn registry(&self) -> &FontRegistry {
        &self.registry
    }

    /// Get the font registry (mutable)
    pub fn registry_mut(&mut self) -> &mut FontRegistry {
        &mut self.registry
    }
}

impl Default for FontSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create default font system")
    }
}
