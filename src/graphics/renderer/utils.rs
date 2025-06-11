// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Renderer utility functions
//!
//! This module contains helper functions and utility methods for the renderer,
//! including pixel access, buffer management, and data retrieval operations.

use crate::graphics::renderer::types::Renderer;

impl Renderer {
    /// Get a read-only reference to a pixel at the specified coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate (must be less than width)
    /// * `y` - Y coordinate (must be less than height)
    ///
    /// # Returns
    /// Optional reference to the pixel value, None if coordinates are out of bounds
    pub(crate) fn get_pixel(&self, x: u32, y: u32) -> Option<&u32> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.framebuffer.get(index)
        } else {
            None
        }
    }

    /// Get a mutable reference to a pixel at the specified coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate (must be less than width)
    /// * `y` - Y coordinate (must be less than height)
    ///
    /// # Returns
    /// Optional mutable reference to the pixel value, None if coordinates are out of bounds
    pub(crate) fn get_pixel_mut(&mut self, x: u32, y: u32) -> Option<&mut u32> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.framebuffer.get_mut(index)
        } else {
            None
        }
    }

    /// Get a read-only reference to the entire framebuffer
    ///
    /// # Returns
    /// Slice containing all pixel data in row-major order
    pub fn get_framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    /// Get the screen dimensions
    ///
    /// # Returns
    /// Tuple containing (width, height) in pixels
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the color depth
    ///
    /// # Returns
    /// Color depth in bits per pixel
    pub fn get_depth(&self) -> u8 {
        self.depth
    }

    /// Calculate the total number of pixels in the framebuffer
    ///
    /// # Returns
    /// Total pixel count (width * height)
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Calculate the framebuffer size in bytes
    ///
    /// # Returns
    /// Size of framebuffer in bytes (assumes 4 bytes per pixel)
    pub fn buffer_size_bytes(&self) -> usize {
        self.pixel_count() * std::mem::size_of::<u32>()
    }

    /// Check if coordinates are within the screen bounds
    ///
    /// # Arguments
    /// * `x` - X coordinate to check
    /// * `y` - Y coordinate to check
    ///
    /// # Returns
    /// true if coordinates are valid, false otherwise
    pub fn is_valid_coordinate(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }
}
