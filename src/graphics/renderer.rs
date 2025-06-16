//! Basic renderer
//!
//! This module provides basic 2D rendering operations for the X11 server.

use super::context::GraphicsContext;
use super::types::*;
use crate::core::error::ServerResult;

/// Basic 2D renderer
pub struct Renderer {
    framebuffer: Vec<u32>,
    width: u32,
    height: u32,
    depth: u8,
}

impl Renderer {
    /// Create a new renderer with default "rx" pattern
    pub fn new(width: u32, height: u32, depth: u8) -> Self {
        let size = (width * height) as usize;
        let mut renderer = Self {
            framebuffer: vec![0; size],
            width,
            height,
            depth,
        };

        // Initialize with "rx" pattern
        renderer.draw_rx_pattern();
        renderer
    }

    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    /// Get mutable framebuffer reference
    pub fn framebuffer_mut(&mut self) -> &mut [u32] {
        &mut self.framebuffer
    }

    /// Get renderer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get color depth
    pub fn depth(&self) -> u8 {
        self.depth
    }

    /// Clear the entire framebuffer
    pub fn clear(&mut self, color: Color) {
        let pixel = color.to_u32();
        self.framebuffer.fill(pixel);
    }

    /// Set a pixel
    pub fn set_pixel(&mut self, x: i16, y: i16, color: Color) -> ServerResult<()> {
        if self.is_point_in_bounds(x, y) {
            let index = (y as u32 * self.width + x as u32) as usize;
            self.framebuffer[index] = color.to_u32();
        }
        Ok(())
    }

    /// Get a pixel
    pub fn get_pixel(&self, x: i16, y: i16) -> Option<Color> {
        if self.is_point_in_bounds(x, y) {
            let index = (y as u32 * self.width + x as u32) as usize;
            Some(Color::from_u32(self.framebuffer[index]))
        } else {
            None
        }
    }

    /// Draw a line using Bresenham's algorithm
    pub fn draw_line(
        &mut self,
        x0: i16,
        y0: i16,
        x1: i16,
        y1: i16,
        gc: &GraphicsContext,
    ) -> ServerResult<()> {
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if self.is_clipped(x0, y0, gc) {
                self.set_pixel(x0, y0, gc.foreground)?;
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
        Ok(())
    }

    /// Draw a rectangle outline
    pub fn draw_rectangle(&mut self, rect: Rectangle, gc: &GraphicsContext) -> ServerResult<()> {
        // Top edge
        self.draw_line(rect.x, rect.y, rect.x + rect.width as i16 - 1, rect.y, gc)?;
        // Bottom edge
        self.draw_line(
            rect.x,
            rect.y + rect.height as i16 - 1,
            rect.x + rect.width as i16 - 1,
            rect.y + rect.height as i16 - 1,
            gc,
        )?;
        // Left edge
        self.draw_line(rect.x, rect.y, rect.x, rect.y + rect.height as i16 - 1, gc)?;
        // Right edge
        self.draw_line(
            rect.x + rect.width as i16 - 1,
            rect.y,
            rect.x + rect.width as i16 - 1,
            rect.y + rect.height as i16 - 1,
            gc,
        )?;
        Ok(())
    }

    /// Fill a rectangle
    pub fn fill_rectangle(&mut self, rect: Rectangle, gc: &GraphicsContext) -> ServerResult<()> {
        for y in rect.y..rect.y + rect.height as i16 {
            for x in rect.x..rect.x + rect.width as i16 {
                if self.is_clipped(x, y, gc) {
                    self.set_pixel(x, y, gc.foreground)?;
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
        width: u16,
        height: u16,
        dst_x: i16,
        dst_y: i16,
        gc: &GraphicsContext,
    ) -> ServerResult<()> {
        // Create temporary buffer to handle overlapping copies
        let mut temp_buffer = Vec::new();

        // Copy source area to temp buffer
        for y in 0..height as i16 {
            for x in 0..width as i16 {
                let src_pixel_x = src_x + x;
                let src_pixel_y = src_y + y;

                if let Some(color) = self.get_pixel(src_pixel_x, src_pixel_y) {
                    temp_buffer.push(color);
                } else {
                    temp_buffer.push(Color::BLACK);
                }
            }
        }

        // Copy from temp buffer to destination
        for y in 0..height as i16 {
            for x in 0..width as i16 {
                let dst_pixel_x = dst_x + x;
                let dst_pixel_y = dst_y + y;
                let index = (y * width as i16 + x) as usize;

                if index < temp_buffer.len() && self.is_clipped(dst_pixel_x, dst_pixel_y, gc) {
                    self.set_pixel(dst_pixel_x, dst_pixel_y, temp_buffer[index])?;
                }
            }
        }

        Ok(())
    }

    /// Draw the default "rx" 4x4 micropixel pattern
    pub fn draw_rx_pattern(&mut self) {
        // Clear to dark teal background
        self.clear(Color::rgb(0x00, 0x40, 0x40));

        // Draw a repeating 8x4 "rx" micropixel pattern across the screen
        let pattern_width = 8;
        let pattern_height = 4;

        for start_y in (0..self.height).step_by(pattern_height) {
            for start_x in (0..self.width).step_by(pattern_width) {
                self.draw_rx_micropixel_tile(start_x as i16, start_y as i16);
            }
        }
    }

    /// Draw a single 8x4 "rx" micropixel tile pattern
    /// Creates an abstract representation of "rx" in just 8x4 pixels
    fn draw_rx_micropixel_tile(&mut self, start_x: i16, start_y: i16) {
        let white = Color::WHITE; // Using black for "r" shape
        let black = Color::BLACK; // Using white for "x" shape

        // 8x4 micropixel pattern representing "r"
        // Pixel layout (colors: W=white, B=black, .=background):
        //   0 1 2 3 4 5 6 7
        // 0 B B B B B B B B
        // 1 B W W W B W B W
        // 2 B W B B B B W B
        // 3 B W B B B W B W
        //
        // Left 2 columns suggest "r" shape, right 2 columns suggest "x" shape

        let pattern = [
            [
                Some(black),
                Some(black),
                Some(black),
                Some(black),
                Some(black),
                Some(black),
                Some(black),
                Some(black),
            ],
            [
                Some(black),
                Some(white),
                Some(white),
                Some(white),
                Some(black),
                Some(white),
                Some(black),
                Some(white),
            ],
            [
                Some(black),
                Some(white),
                Some(black),
                Some(black),
                Some(black),
                Some(black),
                Some(white),
                Some(black),
            ],
            [
                Some(black),
                Some(white),
                Some(black),
                Some(black),
                Some(black),
                Some(white),
                Some(black),
                Some(white),
            ],
        ];

        for (y, row) in pattern.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                if let Some(color) = pixel {
                    let _ = self.set_pixel(start_x + x as i16, start_y + y as i16, color);
                }
            }
        }
    }

    /// Check if point is within bounds
    fn is_point_in_bounds(&self, x: i16, y: i16) -> bool {
        x >= 0 && y >= 0 && x < self.width as i16 && y < self.height as i16
    }

    /// Check if point passes clipping test
    fn is_clipped(&self, x: i16, y: i16, gc: &GraphicsContext) -> bool {
        if !self.is_point_in_bounds(x, y) {
            return false;
        }

        if let Some(clip_rect) = gc.clip_region {
            clip_rect.contains(Point::new(x, y))
        } else {
            true
        }
    }
    /// Resize the framebuffer
    pub fn resize(&mut self, width: u32, height: u32) {
        // Validate dimensions
        if width == 0 || height == 0 {
            tracing::warn!(
                "Attempted to resize renderer with invalid dimensions: {}x{}",
                width,
                height
            );
            return;
        }

        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        self.framebuffer.resize(size, 0);
    }
}
