//! Display Management Module for RXServer
//!
//! This module provides comprehensive display management organized into focused submodules:
//! - `init`: Display initialization and screen setup
//! - `screen`: Screen management, configuration, and resources
//! - `visual`: Visual configuration and color depth management
//! - `framebuffer`: Framebuffer management and pixel operations
//! - `colormap`: Color map management and color operations
//! - `manager`: Main display manager coordinating all display components
//! - `types`: Display-related type definitions and enums

pub mod framebuffer;
pub mod init;
pub mod manager;
pub mod screen;
pub mod types;
pub mod visual;

// Re-export commonly used items for convenience
pub use init::{init_display, DisplayInitConfig};
pub use manager::DisplayManager;
pub use screen::{ScreenManager, ScreenConfig};
pub use types::{ScreenInfo, VisualInfo, DisplaySettings};
pub use visual::{VisualManager, VisualConfig};
pub use framebuffer::{Framebuffer, FramebufferConfig};
