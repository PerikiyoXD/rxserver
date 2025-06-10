//! Basic rendering operations
//!
//! This module provides basic 2D rendering operations for the X server.

use crate::graphics::GraphicsContext;
use crate::protocol::types::*;
use crate::{todo_low, todo_medium, Error, Result};

/// Basic software renderer
pub struct Renderer {
    /// Framebuffer data
    framebuffer: Vec<u32>,
    /// Screen width
    width: u32,
    /// Screen height
    height: u32,
    /// Bits per pixel
    depth: u8,
}

impl Renderer {
    /// Create a new renderer
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

    /// Clear the entire screen with a color
    pub fn clear(&mut self, color: u32) {
        for pixel in &mut self.framebuffer {
            *pixel = color;
        }
    }

    /// Clear a rectangular area
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

    /// Draw a point
    pub fn draw_point(&mut self, x: i16, y: i16, gc: &GraphicsContext) -> Result<()> {
        if gc.point_in_clip_region(x, y) {
            if let Some(pixel) = self.get_pixel_mut(x as u32, y as u32) {
                *pixel = gc.foreground;
            }
        }
        Ok(())
    }

    /// Draw a line using Bresenham's algorithm
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

    /// Fill a rectangle
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

    /// Copy area from one location to another
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

    /// Get a pixel reference (read-only)
    fn get_pixel(&self, x: u32, y: u32) -> Option<&u32> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.framebuffer.get(index)
        } else {
            None
        }
    }

    /// Get a mutable pixel reference
    fn get_pixel_mut(&mut self, x: u32, y: u32) -> Option<&mut u32> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.framebuffer.get_mut(index)
        } else {
            None
        }
    }

    /// Get the framebuffer data
    pub fn get_framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    /// Get screen dimensions
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get color depth
    pub fn get_depth(&self) -> u8 {
        self.depth
    }
}
