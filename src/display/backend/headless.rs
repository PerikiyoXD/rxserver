//! Headless display backend for testing and server environments

use super::traits::{DisplayBackend, FramebufferId};
use super::{BackendCapabilities, BackendType};
use crate::types::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Headless display backend
#[derive(Debug)]
pub struct HeadlessBackend {
    capabilities: BackendCapabilities,
    current_resolution: crate::display::types::Resolution,
    framebuffers: HashMap<FramebufferId, HeadlessFramebuffer>,
    next_framebuffer_id: u32,
    initialized: bool,
}

#[derive(Debug)]
struct HeadlessFramebuffer {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl HeadlessBackend {
    /// Create a new headless backend
    pub fn new() -> Result<Self> {
        Ok(Self {
            capabilities: BackendCapabilities {
                supports_acceleration: false,
                supports_multiple_displays: true,
                max_resolution: crate::display::types::Resolution::new(8192, 8192),
                supported_color_depths: vec![
                    crate::display::types::ColorDepth::Depth8,
                    crate::display::types::ColorDepth::Depth16,
                    crate::display::types::ColorDepth::Depth24,
                    crate::display::types::ColorDepth::Depth32,
                ],
            },
            current_resolution: crate::display::types::Resolution::new(1920, 1080),
            framebuffers: HashMap::new(),
            next_framebuffer_id: 1,
            initialized: false,
        })
    }
}

#[async_trait]
impl DisplayBackend for HeadlessBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Headless
    }

    fn capabilities(&self) -> &BackendCapabilities {
        &self.capabilities
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing headless display backend");
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down headless display backend");
        self.framebuffers.clear();
        self.initialized = false;
        Ok(())
    }

    fn is_available(&self) -> bool {
        true // Headless is always available
    }

    fn name(&self) -> &str {
        "Headless Display Backend"
    }

    async fn set_resolution(
        &mut self,
        resolution: crate::display::types::Resolution,
    ) -> Result<()> {
        tracing::debug!(
            "Setting headless resolution to {}x{}",
            resolution.width,
            resolution.height
        );
        self.current_resolution = resolution;
        Ok(())
    }

    fn current_resolution(&self) -> crate::display::types::Resolution {
        self.current_resolution
    }

    async fn create_framebuffer(&mut self, width: u32, height: u32) -> Result<FramebufferId> {
        let id = FramebufferId::new(self.next_framebuffer_id);
        self.next_framebuffer_id += 1;

        let framebuffer = HeadlessFramebuffer {
            width,
            height,
            data: vec![0; (width * height * 4) as usize], // RGBA
        };

        self.framebuffers.insert(id, framebuffer);
        tracing::debug!(
            "Created headless framebuffer {:?} ({}x{})",
            id,
            width,
            height
        );

        Ok(id)
    }

    async fn destroy_framebuffer(&mut self, id: FramebufferId) -> Result<()> {
        if self.framebuffers.remove(&id).is_some() {
            tracing::debug!("Destroyed headless framebuffer {:?}", id);
        }
        Ok(())
    }

    async fn present(&mut self, framebuffer_id: FramebufferId) -> Result<()> {
        if self.framebuffers.contains_key(&framebuffer_id) {
            tracing::trace!("Presenting headless framebuffer {:?}", framebuffer_id);
            // In headless mode, "presenting" is a no-op
        }
        Ok(())
    }
}
