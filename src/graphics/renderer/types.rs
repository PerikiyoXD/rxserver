// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Renderer type definitions
//!
//! This module defines the core renderer structures and related types
//! for 2D graphics rendering operations.

/// Basic software renderer for 2D graphics operations
///
/// The Renderer provides fundamental 2D drawing capabilities including
/// points, lines, rectangles, and pixel manipulation with framebuffer management.
#[derive(Debug)]
pub struct Renderer {
    /// Framebuffer data storing pixel values
    pub(crate) framebuffer: Vec<u32>,
    /// Screen width in pixels
    pub(crate) width: u32,
    /// Screen height in pixels
    pub(crate) height: u32,
    /// Color depth in bits per pixel
    pub(crate) depth: u8,
}
