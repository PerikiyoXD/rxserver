//! Virtual Display Management for X11 Server
//!
//! This module creates a native window using winit to display the X11 server's output.
//! It provides a virtual display that shows the rendered content of the X11 windows.

use anyhow::Result;

use crate::display::{
    config::DisplayConfig,
    native_display::NativeDisplay,
    types::{Display, DisplayKind},
    virtual_display::VirtualDisplay,
};

pub mod config;
pub mod native_display;
pub mod types;
pub mod virtual_display;
pub mod virtual_display_app;

pub fn create_display(config: DisplayConfig) -> Result<Display> {
    let display: Display = match config.kind {
        DisplayKind::Virtual => Display::Virtual(VirtualDisplay::new(config)),
        DisplayKind::Native => Display::Native(NativeDisplay::new(config)),
    };
    Ok(display)
}
