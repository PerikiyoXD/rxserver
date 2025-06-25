//! Display abstraction layer
//!
//! This module provides display backends and management for the X11 server.

pub mod backend;
pub mod capabilities;
pub mod configuration;
pub mod types;

// Re-export commonly used types
pub use backend::{BackendCapabilities, BackendType, DisplayBackend};
pub use capabilities::DisplayCapabilities;
pub use configuration::DisplayConfig;
pub use types::{ColorDepth, DisplayInfo, RefreshRate, Resolution};

use crate::types::Result;
use std::sync::Arc;

/// Display manager coordinates all display backends
#[derive(Debug)]
pub struct DisplayManager {
    backends: Vec<Arc<dyn DisplayBackend>>,
    active_backend: Option<Arc<dyn DisplayBackend>>,
    config: DisplayConfig,
}

impl DisplayManager {
    /// Create a new display manager
    pub fn new(config: DisplayConfig) -> Self {
        Self {
            backends: Vec::new(),
            active_backend: None,
            config,
        }
    }

    /// Initialize the display manager with available backends
    pub async fn initialize(&mut self) -> Result<()> {
        // Auto-detect and initialize available backends
        self.detect_backends().await?;

        // Select the best backend based on configuration
        self.select_backend().await?;

        Ok(())
    }

    /// Detect available display backends
    async fn detect_backends(&mut self) -> Result<()> {
        // Add headless backend (always available)
        let headless = Arc::new(backend::headless::HeadlessBackend::new()?);
        self.backends.push(headless);

        // TODO: Add other backends based on platform when implemented
        // let software = Arc::new(backend::software::SoftwareBackend::new()?);
        // self.backends.push(software);
        // if let Ok(hardware) = backend::hardware::HardwareBackend::new() {
        //     self.backends.push(Arc::new(hardware));
        // }

        Ok(())
    }

    /// Select the active backend
    async fn select_backend(&mut self) -> Result<()> {
        // Select based on configuration preference
        for backend in &self.backends {
            if backend.backend_type() == self.config.preferred_backend {
                self.active_backend = Some(backend.clone());
                return Ok(());
            }
        }

        // Fallback to first available backend
        if !self.backends.is_empty() {
            self.active_backend = Some(self.backends[0].clone());
        }

        Ok(())
    }

    /// Get the active display backend
    pub fn active_backend(&self) -> Option<&Arc<dyn DisplayBackend>> {
        self.active_backend.as_ref()
    }
}
