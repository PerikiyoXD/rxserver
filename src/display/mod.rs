//! Display Management Module for RXServer
//!
//! This module provides comprehensive display management organized into focused submodules:
//! - `init`: Display initialization and screen setup
//! - `screen`: Screen management, configuration, and resources
//! - `visual`: Visual configuration and color depth management
//! - `framebuffer`: Framebuffer management and pixel operations
//! - `colormap`: Color map management and color operations
//! - `manager`: Main display manager coordinating all display components
//! - `window_renderer`: Software window rendering to actual OS windows
//! - `types`: Display-related type definitions and enums

pub mod framebuffer;
pub mod init;
pub mod manager;
pub mod screen;
pub mod shared_framebuffers;
pub mod types;
pub mod visual;
pub mod window_renderer;

// Re-export commonly used items for convenience
pub use framebuffer::{Framebuffer, FramebufferConfig};
pub use init::{init_display, DisplayInitConfig};
pub use manager::DisplayManager;
pub use screen::{ScreenConfig, ScreenManager};
pub use types::{DisplaySettings, ScreenInfo, VisualInfo};
pub use visual::{VisualConfig, VisualManager};
pub use window_renderer::{WindowRenderer, WindowRendererConfig};

// Re-export SharedFramebuffers
pub use crate::display::shared_framebuffers::SharedFramebuffers;
