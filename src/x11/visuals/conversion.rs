//! Format conversion utilities
//!
//! This module provides utilities for converting between different pixel formats
//! and color representations.

use crate::ByteOrder;
use crate::x11::visuals::formats::PixelFormat;
use crate::x11::visuals::types::{Visual, VisualClass};
use std::fmt::Debug;

/// Color conversion utilities
#[derive(Debug)]
pub struct ColorConverter;

impl ColorConverter {
    /// Convert RGB888 to RGB565
    pub fn rgb888_to_rgb565(red: u8, green: u8, blue: u8) -> u16 {
        let r = (red >> 3) as u16;
        let g = (green >> 2) as u16;
        let b = (blue >> 3) as u16;
        (r << 11) | (g << 5) | b
    }

    /// Convert RGB565 to RGB888
    pub fn rgb565_to_rgb888(pixel: u16) -> (u8, u8, u8) {
        let r = ((pixel >> 11) & 0x1F) as u8;
        let g = ((pixel >> 5) & 0x3F) as u8;
        let b = (pixel & 0x1F) as u8;

        // Scale to full 8-bit range
        let red = (r << 3) | (r >> 2);
        let green = (g << 2) | (g >> 4);
        let blue = (b << 3) | (b >> 2);

        (red, green, blue)
    }

    /// Convert RGB to grayscale using standard luminance formula
    pub fn rgb_to_grayscale(red: u8, green: u8, blue: u8) -> u8 {
        // ITU-R BT.709 luma coefficients
        let luma = 0.2126 * red as f32 + 0.7152 * green as f32 + 0.0722 * blue as f32;
        luma.round() as u8
    }

    /// Convert grayscale to RGB (all components equal)
    pub fn grayscale_to_rgb(gray: u8) -> (u8, u8, u8) {
        (gray, gray, gray)
    }

    /// Apply gamma correction
    pub fn apply_gamma(value: u8, gamma: f32) -> u8 {
        let normalized = value as f32 / 255.0;
        let corrected = normalized.powf(gamma);
        (corrected * 255.0).round() as u8
    }

    /// Remove gamma correction
    pub fn remove_gamma(value: u8, gamma: f32) -> u8 {
        let normalized = value as f32 / 255.0;
        let linear = normalized.powf(1.0 / gamma);
        (linear * 255.0).round() as u8
    }
}

/// Pixel format converter
#[derive(Debug)]
pub struct PixelFormatConverter {
    source_format: PixelFormat,
    target_format: PixelFormat,
}

impl PixelFormatConverter {
    /// Create a new format converter
    pub fn new(source_format: PixelFormat, target_format: PixelFormat) -> Self {
        Self {
            source_format,
            target_format,
        }
    }

    /// Convert a single pixel from source to target format
    pub fn convert_pixel(&self, source_pixel: u32) -> u32 {
        // Extract RGBA from source
        let red = self.source_format.extract_red(source_pixel);
        let green = self.source_format.extract_green(source_pixel);
        let blue = self.source_format.extract_blue(source_pixel);
        let alpha = self.source_format.extract_alpha(source_pixel);

        // Pack into target format
        self.target_format.pack_rgba(red, green, blue, alpha)
    }

    /// Convert a buffer of pixels
    pub fn convert_buffer(
        &self,
        source_data: &[u8],
        target_data: &mut [u8],
        width: u32,
        height: u32,
    ) -> Result<(), ConversionError> {
        let source_bpp = self.source_format.bytes_per_pixel() as usize;
        let target_bpp = self.target_format.bytes_per_pixel() as usize;

        let source_stride = self.source_format.scanline_stride(width) as usize;
        let target_stride = self.target_format.scanline_stride(width) as usize;

        if source_data.len() < source_stride * height as usize {
            return Err(ConversionError::InsufficientSourceData);
        }

        if target_data.len() < target_stride * height as usize {
            return Err(ConversionError::InsufficientTargetSpace);
        }

        for y in 0..height {
            let source_row_offset = y as usize * source_stride;
            let target_row_offset = y as usize * target_stride;

            for x in 0..width {
                let source_pixel_offset = source_row_offset + x as usize * source_bpp;
                let target_pixel_offset = target_row_offset + x as usize * target_bpp;

                // Read source pixel
                let source_pixel = match source_bpp {
                    1 => source_data[source_pixel_offset] as u32,
                    2 => {
                        let bytes = &source_data[source_pixel_offset..source_pixel_offset + 2];
                        match self.source_format.byte_order {
                            ByteOrder::LittleEndian => {
                                u16::from_le_bytes([bytes[0], bytes[1]]) as u32
                            }
                            ByteOrder::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]) as u32,
                        }
                    }
                    3 => {
                        let bytes = &source_data[source_pixel_offset..source_pixel_offset + 3];
                        match self.source_format.byte_order {
                            ByteOrder::LittleEndian => {
                                (bytes[0] as u32)
                                    | ((bytes[1] as u32) << 8)
                                    | ((bytes[2] as u32) << 16)
                            }
                            ByteOrder::BigEndian => {
                                ((bytes[0] as u32) << 16)
                                    | ((bytes[1] as u32) << 8)
                                    | (bytes[2] as u32)
                            }
                        }
                    }
                    4 => {
                        let bytes = &source_data[source_pixel_offset..source_pixel_offset + 4];
                        match self.source_format.byte_order {
                            ByteOrder::LittleEndian => {
                                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                            }
                            ByteOrder::BigEndian => {
                                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                            }
                        }
                    }
                    _ => return Err(ConversionError::UnsupportedFormat),
                };

                // Convert pixel
                let target_pixel = self.convert_pixel(source_pixel);

                // Write target pixel
                match target_bpp {
                    1 => {
                        target_data[target_pixel_offset] = target_pixel as u8;
                    }
                    2 => {
                        let bytes = match self.target_format.byte_order {
                            ByteOrder::LittleEndian => (target_pixel as u16).to_le_bytes(),
                            ByteOrder::BigEndian => (target_pixel as u16).to_be_bytes(),
                        };
                        target_data[target_pixel_offset..target_pixel_offset + 2]
                            .copy_from_slice(&bytes);
                    }
                    3 => {
                        let bytes = match self.target_format.byte_order {
                            ByteOrder::LittleEndian => [
                                target_pixel as u8,
                                (target_pixel >> 8) as u8,
                                (target_pixel >> 16) as u8,
                            ],
                            ByteOrder::BigEndian => [
                                (target_pixel >> 16) as u8,
                                (target_pixel >> 8) as u8,
                                target_pixel as u8,
                            ],
                        };
                        target_data[target_pixel_offset..target_pixel_offset + 3]
                            .copy_from_slice(&bytes);
                    }
                    4 => {
                        let bytes = match self.target_format.byte_order {
                            ByteOrder::LittleEndian => target_pixel.to_le_bytes(),
                            ByteOrder::BigEndian => target_pixel.to_be_bytes(),
                        };
                        target_data[target_pixel_offset..target_pixel_offset + 4]
                            .copy_from_slice(&bytes);
                    }
                    _ => return Err(ConversionError::UnsupportedFormat),
                }
            }
        }

        Ok(())
    }
}

/// Visual converter for converting between different visual types
#[derive(Debug)]
pub struct VisualConverter;

impl VisualConverter {
    /// Check if conversion between visuals is possible
    pub fn can_convert(source: &Visual, target: &Visual) -> bool {
        // Same visual class is always convertible
        if source.class == target.class {
            return true;
        }

        // Check specific conversion possibilities
        match (source.class, target.class) {
            (VisualClass::StaticGray, VisualClass::GrayScale) => true,
            (VisualClass::GrayScale, VisualClass::StaticGray) => true,
            (VisualClass::StaticColor, VisualClass::PseudoColor) => true,
            (VisualClass::PseudoColor, VisualClass::StaticColor) => true,
            (VisualClass::TrueColor, VisualClass::DirectColor) => true,
            (VisualClass::DirectColor, VisualClass::TrueColor) => true,
            (
                VisualClass::StaticGray | VisualClass::GrayScale,
                VisualClass::StaticColor
                | VisualClass::PseudoColor
                | VisualClass::TrueColor
                | VisualClass::DirectColor,
            ) => true,
            (
                VisualClass::StaticColor
                | VisualClass::PseudoColor
                | VisualClass::TrueColor
                | VisualClass::DirectColor,
                VisualClass::StaticGray | VisualClass::GrayScale,
            ) => true,
            (
                VisualClass::StaticColor | VisualClass::PseudoColor,
                VisualClass::TrueColor | VisualClass::DirectColor,
            ) => true,
            (
                VisualClass::TrueColor | VisualClass::DirectColor,
                VisualClass::StaticColor | VisualClass::PseudoColor,
            ) => true,
            (VisualClass::StaticGray, VisualClass::StaticGray) => todo!(),
            (VisualClass::GrayScale, VisualClass::GrayScale) => todo!(),
            (VisualClass::StaticColor, VisualClass::StaticColor) => todo!(),
            (VisualClass::PseudoColor, VisualClass::PseudoColor) => todo!(),
            (VisualClass::TrueColor, VisualClass::TrueColor) => todo!(),
            (VisualClass::DirectColor, VisualClass::DirectColor) => todo!(),
        }
    }

    /// Get the conversion quality between visuals
    pub fn conversion_quality(source: &Visual, target: &Visual) -> ConversionQuality {
        if source == target {
            return ConversionQuality::Perfect;
        }

        // Same class is usually high quality
        if source.class == target.class {
            return ConversionQuality::High;
        }

        match (source.class, target.class) {
            // Lossless conversions
            (VisualClass::StaticGray, VisualClass::GrayScale)
            | (VisualClass::GrayScale, VisualClass::StaticGray)
            | (VisualClass::StaticColor, VisualClass::PseudoColor)
            | (VisualClass::PseudoColor, VisualClass::StaticColor)
            | (VisualClass::TrueColor, VisualClass::DirectColor)
            | (VisualClass::DirectColor, VisualClass::TrueColor) => ConversionQuality::High,

            // Lossy but reasonable conversions
            (VisualClass::StaticGray | VisualClass::GrayScale, _) => ConversionQuality::Medium,
            (_, VisualClass::StaticGray | VisualClass::GrayScale) => ConversionQuality::Low,

            // Other conversions
            _ => ConversionQuality::Medium,
        }
    }
}

/// Conversion quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConversionQuality {
    /// Perfect conversion (no data loss)
    Perfect,
    /// High quality conversion (minimal data loss)
    High,
    /// Medium quality conversion (some data loss)
    Medium,
    /// Low quality conversion (significant data loss)
    Low,
}

/// Conversion errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    /// Insufficient source data
    InsufficientSourceData,
    /// Insufficient target buffer space
    InsufficientTargetSpace,
    /// Unsupported pixel format
    UnsupportedFormat,
    /// Incompatible visual classes
    IncompatibleVisuals,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::InsufficientSourceData => write!(f, "Insufficient source data"),
            ConversionError::InsufficientTargetSpace => {
                write!(f, "Insufficient target buffer space")
            }
            ConversionError::UnsupportedFormat => write!(f, "Unsupported pixel format"),
            ConversionError::IncompatibleVisuals => write!(f, "Incompatible visual classes"),
        }
    }
}

impl std::error::Error for ConversionError {}
