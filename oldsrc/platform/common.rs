//! Common platform abstractions and utilities

use super::{PlatformAbstraction, PlatformError, PlatformInfo};

/// Base platform implementation with common functionality
pub struct BasePlatform {
    info: PlatformInfo,
    initialized: bool,
}

impl BasePlatform {
    pub fn new(info: PlatformInfo) -> Self {
        Self {
            info,
            initialized: false,
        }
    }
}

impl PlatformAbstraction for BasePlatform {
    fn initialize(&mut self) -> Result<(), PlatformError> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!("Initializing platform: {}", self.info.platform);
        self.initialized = true;
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), PlatformError> {
        if !self.initialized {
            return Ok(());
        }

        tracing::info!("Cleaning up platform: {}", self.info.platform);
        self.initialized = false;
        Ok(())
    }

    fn platform_info(&self) -> PlatformInfo {
        self.info.clone()
    }
}

/// Memory allocation utilities
pub mod memory {
    /// Aligned memory allocator for graphics data
    pub struct AlignedAllocator {
        alignment: usize,
    }

    impl AlignedAllocator {
        pub fn new(alignment: usize) -> Self {
            Self { alignment }
        }

        pub fn allocate(&self, size: usize) -> Result<*mut u8, super::PlatformError> {
            let layout =
                std::alloc::Layout::from_size_align(size, self.alignment).map_err(|e| {
                    super::PlatformError::InitializationFailed(format!(
                        "Invalid memory layout: {}",
                        e
                    ))
                })?;

            let ptr = unsafe { std::alloc::alloc(layout) };
            if ptr.is_null() {
                Err(super::PlatformError::InitializationFailed(
                    "Memory allocation failed".to_string(),
                ))
            } else {
                Ok(ptr)
            }
        }

        pub unsafe fn deallocate(&self, ptr: *mut u8, size: usize) {
            let layout = std::alloc::Layout::from_size_align_unchecked(size, self.alignment);
            std::alloc::dealloc(ptr, layout);
        }
    }
}

/// File system utilities
pub mod filesystem {
    use std::path::Path;

    /// Check if a path exists and is accessible
    pub fn is_accessible(path: &Path) -> bool {
        path.exists() && path.metadata().is_ok()
    }

    /// Create directory if it doesn't exist
    pub fn ensure_directory(path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }
}

/// Process utilities
pub mod process {
    /// Get current process ID
    pub fn get_pid() -> u32 {
        std::process::id()
    }

    /// Get current user ID (Unix only)
    #[cfg(unix)]
    pub fn get_uid() -> u32 {
        unsafe { libc::getuid() }
    }

    /// Get current user ID (Windows - always returns 0)
    #[cfg(windows)]
    pub fn get_uid() -> u32 {
        0 // Windows doesn't have Unix-style UIDs
    }
}
