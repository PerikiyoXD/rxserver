// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Renderer library module
//!
//! This module provides a comprehensive 2D software renderer for the RX server,
//! offering fundamental drawing operations including points, lines, rectangles,
//! area copying, and framebuffer management.
//!
//! # Features
//!
//! - Software-based 2D rendering
//! - Bresenham line drawing algorithm
//! - Rectangle drawing and filling
//! - Area copying with overlap handling
//! - Clipping region support via GraphicsContext
//! - Direct framebuffer access
//!
//! # Examples
//!
//! ```rust
//! use crate::graphics::renderer::Renderer;
//! use crate::graphics::GraphicsContext;
//! use crate::protocol::types::Rectangle;
//!
//! // Create a new renderer
//! let mut renderer = Renderer::new(800, 600, 32);
//!
//! // Clear the screen
//! renderer.clear(0x000000); // Black
//!
//! // Draw a rectangle
//! let rect = Rectangle { x: 10, y: 10, width: 100, height: 50 };
//! let gc = GraphicsContext::default();
//! renderer.fill_rectangle(&rect, &gc).unwrap();
//! ```

mod types;
mod rendering;
mod utils;

// Public re-exports
pub use types::Renderer;
