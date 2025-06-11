//! Framebuffer management and pixel operations
//!
//! This module handles framebuffer initialization, pixel format management,
//! and low-level drawing operations for the display system.

use crate::{display::types::FramebufferSettings, Result};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Framebuffer configuration
#[derive(Debug, Clone)]
pub struct FramebufferConfig {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Bits per pixel
    pub bpp: u8,
    /// Bytes per scanline
    pub stride: u32,
    /// Pixel format
    pub format: PixelFormat,
    /// Use software rendering
    pub software: bool,
    /// Scanline padding
    pub scanline_pad: u8,
    /// Byte order (true for little endian)
    pub little_endian: bool,
}

/// Pixel format enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum PixelFormat {
    /// RGB format (24-bit)
    RGB24,
    /// RGBA format (32-bit)
    RGBA32,
    /// BGR format (24-bit)
    BGR24,
    /// BGRA format (32-bit)
    BGRA32,
    /// RGB format (16-bit, 5-6-5)
    RGB565,
    /// RGB format (15-bit, 5-5-5)
    RGB555,
    /// 8-bit indexed color
    Indexed8,
    /// 4-bit indexed color
    Indexed4,
    /// 1-bit monochrome
    Monochrome,
}

/// Framebuffer implementation
pub struct Framebuffer {
    /// Configuration
    config: FramebufferConfig,
    /// Pixel data buffer
    buffer: Arc<Mutex<Vec<u8>>>,
    /// Damage tracking (for optimized updates)
    damage_regions: Arc<Mutex<Vec<DamageRegion>>>,
    /// Framebuffer state
    state: FramebufferState,
}

impl std::fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Framebuffer")
            .field("config", &self.config)
            .field("buffer_len", &self.buffer.lock().unwrap().len())
            .field(
                "damage_regions_count",
                &self.damage_regions.lock().unwrap().len(),
            )
            .field("state", &self.state)
            .finish()
    }
}

/// Damage region for tracking changes
#[derive(Debug, Clone)]
pub struct DamageRegion {
    /// Left coordinate
    pub x: u32,
    /// Top coordinate
    pub y: u32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

/// Framebuffer state
#[derive(Debug, Clone, PartialEq)]
pub enum FramebufferState {
    /// Uninitialized
    Uninitialized,
    /// Ready for use
    Ready,
    /// Error state
    Error(String),
}

impl Framebuffer {
    /// Create a new framebuffer
    pub fn new(config: FramebufferConfig) -> Result<Self> {
        info!(
            "Creating framebuffer: {}x{} @ {} bpp",
            config.width, config.height, config.bpp
        );

        // Validate configuration
        Self::validate_config(&config)?;

        // Calculate buffer size
        let buffer_size = (config.stride * config.height) as usize;
        debug!("Framebuffer buffer size: {} bytes", buffer_size);

        // Create buffer
        let buffer = vec![0u8; buffer_size];

        Ok(Self {
            config,
            buffer: Arc::new(Mutex::new(buffer)),
            damage_regions: Arc::new(Mutex::new(Vec::new())),
            state: FramebufferState::Ready,
        })
    }

    /// Create framebuffer from settings
    pub fn from_settings(width: u32, height: u32, settings: &FramebufferSettings) -> Result<Self> {
        let format = match settings.bpp {
            32 => PixelFormat::RGBA32,
            24 => PixelFormat::RGB24,
            16 => PixelFormat::RGB565,
            8 => PixelFormat::Indexed8,
            4 => PixelFormat::Indexed4,
            1 => PixelFormat::Monochrome,
            _ => {
                return Err(crate::ServerError::ConfigurationError(format!(
                    "Unsupported bits per pixel: {}",
                    settings.bpp
                )))
            }
        };

        let stride = Self::calculate_stride(width, settings.bpp, settings.scanline_pad);

        let config = FramebufferConfig {
            width,
            height,
            bpp: settings.bpp,
            stride,
            format,
            software: settings.software,
            scanline_pad: settings.scanline_pad,
            little_endian: settings.little_endian,
        };

        Self::new(config)
    }

    /// Get framebuffer configuration
    pub fn config(&self) -> &FramebufferConfig {
        &self.config
    }

    /// Get framebuffer state
    pub fn state(&self) -> &FramebufferState {
        &self.state
    }

    /// Get buffer size in bytes
    pub fn buffer_size(&self) -> usize {
        (self.config.stride * self.config.height) as usize
    }

    /// Clear the framebuffer
    pub fn clear(&self, color: u32) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();

        match self.config.format {
            PixelFormat::RGBA32 | PixelFormat::BGRA32 => {
                let pixel_bytes = color.to_le_bytes();
                for chunk in buffer.chunks_exact_mut(4) {
                    chunk.copy_from_slice(&pixel_bytes);
                }
            }
            PixelFormat::RGB24 | PixelFormat::BGR24 => {
                let pixel_bytes = [
                    (color & 0xFF) as u8,
                    ((color >> 8) & 0xFF) as u8,
                    ((color >> 16) & 0xFF) as u8,
                ];
                for chunk in buffer.chunks_exact_mut(3) {
                    chunk.copy_from_slice(&pixel_bytes);
                }
            }
            PixelFormat::RGB565 | PixelFormat::RGB555 => {
                let pixel_bytes = (color as u16).to_le_bytes();
                for chunk in buffer.chunks_exact_mut(2) {
                    chunk.copy_from_slice(&pixel_bytes);
                }
            }
            PixelFormat::Indexed8 => {
                buffer.fill(color as u8);
            }
            PixelFormat::Indexed4 => {
                let packed_color = ((color as u8) & 0x0F) | (((color as u8) & 0x0F) << 4);
                buffer.fill(packed_color);
            }
            PixelFormat::Monochrome => {
                buffer.fill(if color != 0 { 0xFF } else { 0x00 });
            }
        }

        // Mark entire framebuffer as damaged
        self.add_damage_region(DamageRegion {
            x: 0,
            y: 0,
            width: self.config.width,
            height: self.config.height,
        });

        debug!("Cleared framebuffer with color 0x{:08X}", color);
        Ok(())
    }

    /// Set a pixel at the given coordinates
    pub fn set_pixel(&self, x: u32, y: u32, color: u32) -> Result<()> {
        if x >= self.config.width || y >= self.config.height {
            return Err(crate::ServerError::InvalidParameter(format!(
                "Pixel coordinates ({}, {}) out of bounds",
                x, y
            )));
        }

        let mut buffer = self.buffer.lock().unwrap();
        let offset = (y * self.config.stride + x * (self.config.bpp as u32 / 8)) as usize;

        match self.config.format {
            PixelFormat::RGBA32 => {
                if offset + 4 <= buffer.len() {
                    let pixel_bytes = color.to_le_bytes();
                    buffer[offset..offset + 4].copy_from_slice(&pixel_bytes);
                }
            }
            PixelFormat::BGRA32 => {
                if offset + 4 <= buffer.len() {
                    let pixel_bytes = [
                        (color & 0xFF) as u8,         // B
                        ((color >> 8) & 0xFF) as u8,  // G
                        ((color >> 16) & 0xFF) as u8, // R
                        ((color >> 24) & 0xFF) as u8, // A
                    ];
                    buffer[offset..offset + 4].copy_from_slice(&pixel_bytes);
                }
            }
            PixelFormat::RGB24 => {
                if offset + 3 <= buffer.len() {
                    buffer[offset] = ((color >> 16) & 0xFF) as u8; // R
                    buffer[offset + 1] = ((color >> 8) & 0xFF) as u8; // G
                    buffer[offset + 2] = (color & 0xFF) as u8; // B
                }
            }
            PixelFormat::BGR24 => {
                if offset + 3 <= buffer.len() {
                    buffer[offset] = (color & 0xFF) as u8; // B
                    buffer[offset + 1] = ((color >> 8) & 0xFF) as u8; // G
                    buffer[offset + 2] = ((color >> 16) & 0xFF) as u8; // R
                }
            }
            PixelFormat::RGB565 => {
                if offset + 2 <= buffer.len() {
                    let pixel =
                        ((color >> 8) & 0xF800) | ((color >> 5) & 0x07E0) | ((color >> 3) & 0x001F);
                    let pixel_bytes = (pixel as u16).to_le_bytes();
                    buffer[offset..offset + 2].copy_from_slice(&pixel_bytes);
                }
            }
            PixelFormat::RGB555 => {
                if offset + 2 <= buffer.len() {
                    let pixel =
                        ((color >> 9) & 0x7C00) | ((color >> 6) & 0x03E0) | ((color >> 3) & 0x001F);
                    let pixel_bytes = (pixel as u16).to_le_bytes();
                    buffer[offset..offset + 2].copy_from_slice(&pixel_bytes);
                }
            }
            PixelFormat::Indexed8 => {
                if offset < buffer.len() {
                    buffer[offset] = color as u8;
                }
            }
            PixelFormat::Indexed4 => {
                if offset < buffer.len() {
                    let byte_offset = offset / 2;
                    let nibble_offset = offset % 2;
                    if byte_offset < buffer.len() {
                        if nibble_offset == 0 {
                            buffer[byte_offset] =
                                (buffer[byte_offset] & 0xF0) | ((color as u8) & 0x0F);
                        } else {
                            buffer[byte_offset] =
                                (buffer[byte_offset] & 0x0F) | (((color as u8) & 0x0F) << 4);
                        }
                    }
                }
            }
            PixelFormat::Monochrome => {
                let byte_offset = offset / 8;
                let bit_offset = offset % 8;
                if byte_offset < buffer.len() {
                    if color != 0 {
                        buffer[byte_offset] |= 1 << bit_offset;
                    } else {
                        buffer[byte_offset] &= !(1 << bit_offset);
                    }
                }
            }
        }

        // Add damage region for this pixel
        self.add_damage_region(DamageRegion {
            x,
            y,
            width: 1,
            height: 1,
        });

        Ok(())
    }

    /// Get a pixel value at the given coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Result<u32> {
        if x >= self.config.width || y >= self.config.height {
            return Err(crate::ServerError::InvalidParameter(format!(
                "Pixel coordinates ({}, {}) out of bounds",
                x, y
            )));
        }

        let buffer = self.buffer.lock().unwrap();
        let offset = (y * self.config.stride + x * (self.config.bpp as u32 / 8)) as usize;

        let color = match self.config.format {
            PixelFormat::RGBA32 => {
                if offset + 4 <= buffer.len() {
                    u32::from_le_bytes([
                        buffer[offset],
                        buffer[offset + 1],
                        buffer[offset + 2],
                        buffer[offset + 3],
                    ])
                } else {
                    0
                }
            }
            PixelFormat::BGRA32 => {
                if offset + 4 <= buffer.len() {
                    // Convert BGRA to RGBA
                    let b = buffer[offset] as u32;
                    let g = buffer[offset + 1] as u32;
                    let r = buffer[offset + 2] as u32;
                    let a = buffer[offset + 3] as u32;
                    (a << 24) | (r << 16) | (g << 8) | b
                } else {
                    0
                }
            }
            PixelFormat::RGB24 => {
                if offset + 3 <= buffer.len() {
                    let r = buffer[offset] as u32;
                    let g = buffer[offset + 1] as u32;
                    let b = buffer[offset + 2] as u32;
                    0xFF000000 | (r << 16) | (g << 8) | b
                } else {
                    0
                }
            }
            PixelFormat::BGR24 => {
                if offset + 3 <= buffer.len() {
                    let b = buffer[offset] as u32;
                    let g = buffer[offset + 1] as u32;
                    let r = buffer[offset + 2] as u32;
                    0xFF000000 | (r << 16) | (g << 8) | b
                } else {
                    0
                }
            }
            PixelFormat::RGB565 => {
                if offset + 2 <= buffer.len() {
                    let pixel = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]) as u32;
                    let r = ((pixel & 0xF800) >> 8) as u32;
                    let g = ((pixel & 0x07E0) >> 3) as u32;
                    let b = ((pixel & 0x001F) << 3) as u32;
                    0xFF000000 | (r << 16) | (g << 8) | b
                } else {
                    0
                }
            }
            PixelFormat::RGB555 => {
                if offset + 2 <= buffer.len() {
                    let pixel = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]) as u32;
                    let r = ((pixel & 0x7C00) >> 7) as u32;
                    let g = ((pixel & 0x03E0) >> 2) as u32;
                    let b = ((pixel & 0x001F) << 3) as u32;
                    0xFF000000 | (r << 16) | (g << 8) | b
                } else {
                    0
                }
            }
            PixelFormat::Indexed8 => {
                if offset < buffer.len() {
                    // For indexed color, we'd need a color palette
                    // For now, treat as grayscale
                    let gray = buffer[offset] as u32;
                    0xFF000000 | (gray << 16) | (gray << 8) | gray
                } else {
                    0
                }
            }
            PixelFormat::Indexed4 => {
                if offset < buffer.len() {
                    let byte_offset = offset / 2;
                    let nibble_offset = offset % 2;
                    if byte_offset < buffer.len() {
                        let nibble = if nibble_offset == 0 {
                            buffer[byte_offset] & 0x0F
                        } else {
                            (buffer[byte_offset] & 0xF0) >> 4
                        };
                        let gray = (nibble * 17) as u32; // Scale 4-bit to 8-bit
                        0xFF000000 | (gray << 16) | (gray << 8) | gray
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            PixelFormat::Monochrome => {
                let byte_offset = offset / 8;
                let bit_offset = offset % 8;
                if byte_offset < buffer.len() {
                    let bit = (buffer[byte_offset] >> bit_offset) & 1;
                    if bit != 0 {
                        0xFFFFFFFF // White
                    } else {
                        0xFF000000 // Black
                    }
                } else {
                    0
                }
            }
        };

        Ok(color)
    }

    /// Get access to the raw buffer for reading (unsafe but efficient)
    /// Returns a copy of the buffer data in RGBA32 format for rendering
    pub fn get_rgba32_buffer(&self) -> Vec<u32> {
        let buffer = self.buffer.lock().unwrap();
        let mut rgba_buffer = Vec::with_capacity((self.config.width * self.config.height) as usize);

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                match self.get_pixel(x, y) {
                    Ok(pixel) => rgba_buffer.push(pixel),
                    Err(_) => rgba_buffer.push(0xFF000000), // Black on error
                }
            }
        }

        rgba_buffer
    }

    /// Copy framebuffer content to an RGBA32 buffer (more efficient than pixel-by-pixel)
    pub fn copy_to_rgba32_buffer(&self, dest_buffer: &mut [u32]) -> Result<()> {
        let expected_size = (self.config.width * self.config.height) as usize;
        if dest_buffer.len() < expected_size {
            return Err(crate::ServerError::InvalidParameter(format!(
                "Destination buffer too small: {} < {}",
                dest_buffer.len(),
                expected_size
            )));
        }

        let buffer = self.buffer.lock().unwrap();

        match self.config.format {
            PixelFormat::RGBA32 => {
                // Direct copy for RGBA32 format
                for y in 0..self.config.height {
                    let src_row_start = (y * self.config.stride) as usize;
                    let dest_row_start = (y * self.config.width) as usize;

                    for x in 0..self.config.width {
                        let src_offset = src_row_start + (x * 4) as usize;
                        let dest_offset = dest_row_start + x as usize;

                        if src_offset + 4 <= buffer.len() && dest_offset < dest_buffer.len() {
                            dest_buffer[dest_offset] = u32::from_le_bytes([
                                buffer[src_offset],
                                buffer[src_offset + 1],
                                buffer[src_offset + 2],
                                buffer[src_offset + 3],
                            ]);
                        }
                    }
                }
            }
            _ => {
                // Convert pixel by pixel for other formats
                for y in 0..self.config.height {
                    for x in 0..self.config.width {
                        let dest_offset = (y * self.config.width + x) as usize;
                        if dest_offset < dest_buffer.len() {
                            match self.get_pixel(x, y) {
                                Ok(pixel) => dest_buffer[dest_offset] = pixel,
                                Err(_) => dest_buffer[dest_offset] = 0xFF000000,
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get damage regions and clear the list
    pub fn get_and_clear_damage(&self) -> Vec<DamageRegion> {
        let mut damage = self.damage_regions.lock().unwrap();
        let regions = damage.clone();
        damage.clear();
        regions
    }

    /// Add a damage region
    fn add_damage_region(&self, region: DamageRegion) {
        let mut damage = self.damage_regions.lock().unwrap();
        damage.push(region);
    }

    /// Calculate stride for given parameters
    fn calculate_stride(width: u32, bpp: u8, scanline_pad: u8) -> u32 {
        let bits_per_line = width * bpp as u32;
        let bytes_per_line = (bits_per_line + 7) / 8;
        let pad_mask = (scanline_pad as u32) - 1;
        (bytes_per_line + pad_mask) & !pad_mask
    }

    /// Validate framebuffer configuration
    fn validate_config(config: &FramebufferConfig) -> Result<()> {
        if config.width == 0 || config.height == 0 {
            return Err(crate::ServerError::ConfigurationError(
                "Framebuffer dimensions must be greater than 0".to_string(),
            ));
        }

        if ![1, 4, 8, 16, 24, 32].contains(&config.bpp) {
            return Err(crate::ServerError::ConfigurationError(format!(
                "Unsupported bits per pixel: {}",
                config.bpp
            )));
        }

        if ![1, 2, 4, 8, 16, 32].contains(&config.scanline_pad) {
            return Err(crate::ServerError::ConfigurationError(format!(
                "Invalid scanline padding: {}",
                config.scanline_pad
            )));
        }

        // Validate stride
        let min_stride = (config.width * config.bpp as u32 + 7) / 8;
        if config.stride < min_stride {
            return Err(crate::ServerError::ConfigurationError(format!(
                "Stride {} too small for width {} and bpp {}",
                config.stride, config.width, config.bpp
            )));
        }

        Ok(())
    }

    /// Resize the framebuffer
    pub fn resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        info!(
            "Resizing framebuffer from {}x{} to {}x{}",
            self.config.width, self.config.height, new_width, new_height
        );

        // Update configuration
        self.config.width = new_width;
        self.config.height = new_height;
        self.config.stride =
            Self::calculate_stride(new_width, self.config.bpp, self.config.scanline_pad);

        // Recreate buffer
        let buffer_size = (self.config.stride * self.config.height) as usize;
        let mut buffer = self.buffer.lock().unwrap();
        buffer.resize(buffer_size, 0);

        // Clear damage regions
        let mut damage = self.damage_regions.lock().unwrap();
        damage.clear();

        debug!("Framebuffer resized successfully");
        Ok(())
    }
}

impl PixelFormat {
    /// Get bytes per pixel for this format
    pub fn bytes_per_pixel(&self) -> u8 {
        match self {
            PixelFormat::RGB24 | PixelFormat::BGR24 => 3,
            PixelFormat::RGBA32 | PixelFormat::BGRA32 => 4,
            PixelFormat::RGB565 | PixelFormat::RGB555 => 2,
            PixelFormat::Indexed8 => 1,
            PixelFormat::Indexed4 => 1, // Actually 0.5, but we handle packing separately
            PixelFormat::Monochrome => 1, // Actually 0.125, but we handle bit packing separately
        }
    }

    /// Get bits per pixel for this format
    pub fn bits_per_pixel(&self) -> u8 {
        match self {
            PixelFormat::RGB24 | PixelFormat::BGR24 => 24,
            PixelFormat::RGBA32 | PixelFormat::BGRA32 => 32,
            PixelFormat::RGB565 | PixelFormat::RGB555 => 16,
            PixelFormat::Indexed8 => 8,
            PixelFormat::Indexed4 => 4,
            PixelFormat::Monochrome => 1,
        }
    }
}

impl From<&FramebufferSettings> for PixelFormat {
    fn from(settings: &FramebufferSettings) -> Self {
        match settings.bpp {
            32 => PixelFormat::RGBA32,
            24 => PixelFormat::RGB24,
            16 => PixelFormat::RGB565,
            8 => PixelFormat::Indexed8,
            4 => PixelFormat::Indexed4,
            1 => PixelFormat::Monochrome,
            _ => PixelFormat::RGBA32, // Default fallback
        }
    }
}
