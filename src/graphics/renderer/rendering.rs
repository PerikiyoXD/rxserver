// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Core rendering implementations
//!
//! This module contains the actual rendering algorithms and drawing operations
//! for the software renderer, including line drawing, rectangles, and pixel operations.

use crate::graphics::renderer::types::Renderer;
use crate::graphics::GraphicsContext;
use crate::protocol::types::*;
use crate::{Error, Result};

impl Renderer {
    /// Create a new renderer with specified dimensions and color depth
    ///
    /// # Arguments
    /// * `width` - Screen width in pixels
    /// * `height` - Screen height in pixels  
    /// * `depth` - Color depth in bits per pixel
    ///
    /// # Returns
    /// A new Renderer instance with allocated framebuffer
    pub fn new(width: u32, height: u32, depth: u8) -> Self {
        let pixel_count = (width * height) as usize;
        let framebuffer = vec![0; pixel_count];

        Self {
            framebuffer,
            width,
            height,
            depth,
        }
    }

    /// Clear the entire screen with a solid color
    ///
    /// # Arguments
    /// * `color` - 32-bit color value to fill the screen with
    pub fn clear(&mut self, color: u32) {
        for pixel in &mut self.framebuffer {
            *pixel = color;
        }
    }

    /// Clear a rectangular area with a solid color
    ///
    /// # Arguments
    /// * `rect` - Rectangle defining the area to clear
    /// * `color` - 32-bit color value to fill the area with
    pub fn clear_area(&mut self, rect: &Rectangle, color: u32) {
        let x_end = (rect.x + rect.width as i16).min(self.width as i16);
        let y_end = (rect.y + rect.height as i16).min(self.height as i16);

        for y in rect.y.max(0)..y_end {
            for x in rect.x.max(0)..x_end {
                if let Some(pixel) = self.get_pixel_mut(x as u32, y as u32) {
                    *pixel = color;
                }
            }
        }
    }

    /// Draw a single point at the specified coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `gc` - Graphics context containing drawing parameters
    pub fn draw_point(&mut self, x: i16, y: i16, gc: &GraphicsContext) -> Result<()> {
        if gc.point_in_clip_region(x, y) {
            if let Some(pixel) = self.get_pixel_mut(x as u32, y as u32) {
                *pixel = gc.foreground;
            }
        }
        Ok(())
    }

    /// Draw a line between two points using Bresenham's algorithm
    ///
    /// # Arguments
    /// * `x1`, `y1` - Starting point coordinates
    /// * `x2`, `y2` - Ending point coordinates
    /// * `gc` - Graphics context containing drawing parameters
    pub fn draw_line(
        &mut self,
        x1: i16,
        y1: i16,
        x2: i16,
        y2: i16,
        gc: &GraphicsContext,
    ) -> Result<()> {
        let mut x = x1;
        let mut y = y1;
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if gc.point_in_clip_region(x, y) {
                if let Some(pixel) = self.get_pixel_mut(x as u32, y as u32) {
                    *pixel = gc.foreground;
                }
            }

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }

        Ok(())
    }

    /// Draw a rectangle outline
    ///
    /// # Arguments
    /// * `rect` - Rectangle to draw
    /// * `gc` - Graphics context containing drawing parameters
    pub fn draw_rectangle(&mut self, rect: &Rectangle, gc: &GraphicsContext) -> Result<()> {
        if !gc.rect_in_clip_region(rect) {
            return Ok(());
        }

        let x2 = rect.x + rect.width as i16 - 1;
        let y2 = rect.y + rect.height as i16 - 1;

        // Draw the four sides
        self.draw_line(rect.x, rect.y, x2, rect.y, gc)?; // Top
        self.draw_line(x2, rect.y, x2, y2, gc)?; // Right
        self.draw_line(x2, y2, rect.x, y2, gc)?; // Bottom
        self.draw_line(rect.x, y2, rect.x, rect.y, gc)?; // Left

        Ok(())
    }

    /// Fill a rectangle with a solid color
    ///
    /// # Arguments
    /// * `rect` - Rectangle to fill
    /// * `gc` - Graphics context containing drawing parameters
    pub fn fill_rectangle(&mut self, rect: &Rectangle, gc: &GraphicsContext) -> Result<()> {
        if !gc.rect_in_clip_region(rect) {
            return Ok(());
        }

        let x_end = (rect.x + rect.width as i16).min(self.width as i16);
        let y_end = (rect.y + rect.height as i16).min(self.height as i16);

        for y in rect.y.max(0)..y_end {
            for x in rect.x.max(0)..x_end {
                if gc.point_in_clip_region(x, y) {
                    if let Some(pixel) = self.get_pixel_mut(x as u32, y as u32) {
                        *pixel = gc.foreground;
                    }
                }
            }
        }

        Ok(())
    }

    /// Copy a rectangular area from one location to another
    ///
    /// # Arguments
    /// * `src_x`, `src_y` - Source area top-left coordinates
    /// * `dst_x`, `dst_y` - Destination area top-left coordinates
    /// * `width`, `height` - Dimensions of area to copy
    /// * `gc` - Graphics context containing drawing parameters
    pub fn copy_area(
        &mut self,
        src_x: i16,
        src_y: i16,
        dst_x: i16,
        dst_y: i16,
        width: u16,
        height: u16,
        gc: &GraphicsContext,
    ) -> Result<()> {
        // Create a temporary buffer to handle overlapping copies
        let mut temp_buffer = Vec::new();

        // Copy source data to temporary buffer
        for y in 0..height {
            for x in 0..width {
                let src_pixel_x = src_x + x as i16;
                let src_pixel_y = src_y + y as i16;

                if let Some(pixel) = self.get_pixel(src_pixel_x as u32, src_pixel_y as u32) {
                    temp_buffer.push(*pixel);
                } else {
                    temp_buffer.push(0);
                }
            }
        }

        // Copy from temporary buffer to destination
        for y in 0..height {
            for x in 0..width {
                let dst_pixel_x = dst_x + x as i16;
                let dst_pixel_y = dst_y + y as i16;

                if gc.point_in_clip_region(dst_pixel_x, dst_pixel_y) {
                    if let Some(pixel) = self.get_pixel_mut(dst_pixel_x as u32, dst_pixel_y as u32)
                    {
                        let temp_index = (y as usize * width as usize) + x as usize;
                        if temp_index < temp_buffer.len() {
                            *pixel = temp_buffer[temp_index];
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
