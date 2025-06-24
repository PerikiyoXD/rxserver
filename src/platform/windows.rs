//! Windows platform support

use super::{Platform, PlatformAbstraction, PlatformCapabilities, PlatformError, PlatformInfo};

/// Windows platform implementation
pub struct WindowsPlatform {
    capabilities: PlatformCapabilities,
}

impl WindowsPlatform {
    /// Create new Windows platform
    pub fn new() -> Result<Self, PlatformError> {
        Ok(Self {
            capabilities: Platform::Windows.capabilities(),
        })
    }

    /// Get platform capabilities
    pub fn capabilities(&self) -> &PlatformCapabilities {
        &self.capabilities
    }
}

impl PlatformAbstraction for WindowsPlatform {
    /// Initialize platform-specific resources
    fn initialize(&mut self) -> Result<(), PlatformError> {
        // Windows-specific initialization
        Ok(())
    }

    /// Cleanup platform-specific resources
    fn cleanup(&mut self) -> Result<(), PlatformError> {
        // Windows-specific cleanup
        Ok(())
    }

    /// Get platform information
    fn platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            platform: Platform::Windows,
            version: super::get_platform_version(),
            architecture: super::get_architecture(),
            capabilities: self.capabilities.clone(),
        }
    }
}
