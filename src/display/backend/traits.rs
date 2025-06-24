//! Display backend trait definitions

use super::{BackendCapabilities, BackendType};
use crate::types::Result;
use async_trait::async_trait;

/// Main display backend trait
#[async_trait]
pub trait DisplayBackend: Send + Sync + std::fmt::Debug {
    /// Get the backend type
    fn backend_type(&self) -> BackendType;

    /// Get backend capabilities
    fn capabilities(&self) -> &BackendCapabilities;

    /// Initialize the backend
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the backend
    async fn shutdown(&mut self) -> Result<()>;

    /// Check if backend is available on this system
    fn is_available(&self) -> bool;

    /// Get backend name/description
    fn name(&self) -> &str;

    /// Set display resolution
    async fn set_resolution(&mut self, resolution: crate::display::types::Resolution)
    -> Result<()>;

    /// Get current resolution
    fn current_resolution(&self) -> crate::display::types::Resolution;

    /// Create a new framebuffer
    async fn create_framebuffer(&mut self, width: u32, height: u32) -> Result<FramebufferId>;

    /// Destroy a framebuffer
    async fn destroy_framebuffer(&mut self, id: FramebufferId) -> Result<()>;

    /// Present/flip framebuffer to display
    async fn present(&mut self, framebuffer_id: FramebufferId) -> Result<()>;
}

/// Framebuffer identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FramebufferId(pub u32);

impl FramebufferId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}
