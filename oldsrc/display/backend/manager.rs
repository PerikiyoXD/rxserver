//! Display backend manager

use super::{BackendType, DisplayBackend};
use crate::types::Result;
use std::sync::Arc;

/// Backend manager for display backends
#[derive(Debug)]
pub struct BackendManager {
    backends: Vec<Arc<dyn DisplayBackend>>,
    active_backend: Option<Arc<dyn DisplayBackend>>,
}

impl BackendManager {
    /// Create new backend manager
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            active_backend: None,
        }
    }

    /// Initialize available backends
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing display backends"); // Add headless backend (always available)
        let headless = super::headless::HeadlessBackend::new()?;
        self.backends.push(Arc::new(headless));

        Ok(())
    }

    /// Select best backend
    pub fn select_backend(&mut self, preferred: BackendType) -> Result<()> {
        for backend in &self.backends {
            if backend.backend_type() == preferred {
                self.active_backend = Some(backend.clone());
                return Ok(());
            }
        }

        // Fallback to first available backend
        if let Some(backend) = self.backends.first() {
            self.active_backend = Some(backend.clone());
        }

        Ok(())
    }

    /// Get active backend
    pub fn active_backend(&self) -> Option<&Arc<dyn DisplayBackend>> {
        self.active_backend.as_ref()
    }
}
