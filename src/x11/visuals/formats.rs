//! Pixel format management
//!
//! This module handles pixel format definitions and conversions between different
//! pixel representations.

use crate::x11::protocol::endianness::ByteOrder;
use crate::x11::visuals::depth::ColorDepth;
use crate::x11::visuals::types::{Visual, VisualClass};

/// Pixel format descriptor
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelFormat {
    /// Bits per pixel
    pub bits_per_pixel: u8,
    /// Color depth
    pub depth: ColorDepth,
    /// Red field information
    pub red_field: ColorField,
    /// Green field information
    pub green_field: ColorField,
    /// Blue field information
    pub blue_field: ColorField,
    /// Alpha field information (if present)
    pub alpha_field: Option<ColorField>,
    /// Byte order
    pub byte_order: ByteOrder,
    /// Scanline pad
    pub scanline_pad: u8,
}

/// Color field descriptor
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorField {
    /// Bit offset within pixel
    pub offset: u8,
    /// Number of bits for this field
    pub size: u8,
}

impl PixelFormat {
    /// Create a new pixel format
    pub fn new(
        bits_per_pixel: u8,
        depth: ColorDepth,
        red_field: ColorField,
        green_field: ColorField,
        blue_field: ColorField,
        alpha_field: Option<ColorField>,
        byte_order: ByteOrder,
        scanline_pad: u8,
    ) -> Self {
        Self {
            bits_per_pixel,
            depth,
            red_field,
            green_field,
            blue_field,
            alpha_field,
            byte_order,
            scanline_pad,
        }
    }

    /// Create a pixel format from a visual
    pub fn from_visual(visual: &Visual, depth: ColorDepth) -> Self {
        let bits_per_pixel = match visual.class {
            VisualClass::TrueColor | VisualClass::DirectColor => {
                // Calculate from masks
                let max_bit = [visual.red_mask, visual.green_mask, visual.blue_mask]
                    .iter()
                    .map(|mask| {
                        if *mask == 0 {
                            0
                        } else {
                            32 - mask.leading_zeros()
                        }
                    })
                    .max()
                    .unwrap_or(0) as u8;

                // Round up to common bit depths
                match max_bit {
                    0..=8 => 8,
                    9..=16 => 16,
                    17..=24 => 24,
                    25..=32 => 32,
                    _ => 32,
                }
            }
            _ => depth.value(),
        };

        let (red_field, green_field, blue_field) = if visual.class.is_direct_color() {
            (
                ColorField::from_mask(visual.red_mask),
                ColorField::from_mask(visual.green_mask),
                ColorField::from_mask(visual.blue_mask),
            )
        } else {
            // For indexed color, no RGB fields
            (
                ColorField { offset: 0, size: 0 },
                ColorField { offset: 0, size: 0 },
                ColorField { offset: 0, size: 0 },
            )
        };

        Self {
            bits_per_pixel,
            depth,
            red_field,
            green_field,
            blue_field,
            alpha_field: None, // X11 doesn't typically use alpha in visuals
            byte_order: ByteOrder::LittleEndian, // Default to little endian
            scanline_pad: 32,  // Common scanline padding
        }
    }

    /// Get the number of bytes per pixel
    pub fn bytes_per_pixel(&self) -> u8 {
        (self.bits_per_pixel + 7) / 8
    }

    /// Check if this format has an alpha channel
    pub fn has_alpha(&self) -> bool {
        self.alpha_field.is_some()
    }

    /// Extract red component from pixel
    pub fn extract_red(&self, pixel: u32) -> u8 {
        self.red_field.extract(pixel)
    }

    /// Extract green component from pixel
    pub fn extract_green(&self, pixel: u32) -> u8 {
        self.green_field.extract(pixel)
    }

    /// Extract blue component from pixel
    pub fn extract_blue(&self, pixel: u32) -> u8 {
        self.blue_field.extract(pixel)
    }

    /// Extract alpha component from pixel
    pub fn extract_alpha(&self, pixel: u32) -> u8 {
        if let Some(alpha_field) = &self.alpha_field {
            alpha_field.extract(pixel)
        } else {
            255 // Fully opaque
        }
    }

    /// Pack RGB(A) components into pixel
    pub fn pack_rgba(&self, red: u8, green: u8, blue: u8, alpha: u8) -> u32 {
        let mut pixel = 0;
        pixel |= self.red_field.pack(red);
        pixel |= self.green_field.pack(green);
        pixel |= self.blue_field.pack(blue);

        if let Some(alpha_field) = &self.alpha_field {
            pixel |= alpha_field.pack(alpha);
        }

        pixel
    }

    /// Calculate scanline stride for a given width
    pub fn scanline_stride(&self, width: u32) -> u32 {
        let bits_per_line = width * self.bits_per_pixel as u32;
        let pad_bits = self.scanline_pad as u32;
        ((bits_per_line + pad_bits - 1) / pad_bits) * pad_bits / 8
    }
}

impl ColorField {
    /// Create a color field from a bitmask
    pub fn from_mask(mask: u32) -> Self {
        if mask == 0 {
            Self { offset: 0, size: 0 }
        } else {
            let offset = mask.trailing_zeros() as u8;
            let size = (32 - (mask >> offset).leading_zeros()) as u8;
            Self { offset, size }
        }
    }

    /// Get the bitmask for this field
    pub fn mask(&self) -> u32 {
        if self.size == 0 {
            0
        } else {
            let field_mask = (1u32 << self.size) - 1;
            field_mask << self.offset
        }
    }

    /// Extract value from pixel using this field
    pub fn extract(&self, pixel: u32) -> u8 {
        if self.size == 0 {
            0
        } else {
            let mask = self.mask();
            let shifted = (pixel & mask) >> self.offset;
            let max_value = (1u32 << self.size) - 1;
            ((shifted * 255) / max_value) as u8
        }
    }

    /// Pack value into pixel using this field
    pub fn pack(&self, value: u8) -> u32 {
        if self.size == 0 {
            0
        } else {
            let max_value = (1u32 << self.size) - 1;
            let scaled = ((value as u32) * max_value) / 255;
            (scaled << self.offset) & self.mask()
        }
    }
}

/// Manages pixel formats for the display system
#[derive(Debug)]
pub struct FormatManager {
    /// Available formats by depth
    formats: std::collections::HashMap<ColorDepth, Vec<PixelFormat>>,
}

impl FormatManager {
    /// Create a new format manager
    pub fn new() -> Self {
        let mut manager = Self {
            formats: std::collections::HashMap::new(),
        };

        manager.initialize_standard_formats();
        manager
    }

    /// Initialize standard pixel formats
    fn initialize_standard_formats(&mut self) {
        // 8-bit indexed
        let format_8 = PixelFormat::new(
            8,
            ColorDepth::new(8),
            ColorField { offset: 0, size: 0 },
            ColorField { offset: 0, size: 0 },
            ColorField { offset: 0, size: 0 },
            None,
            ByteOrder::LittleEndian,
            32,
        );
        self.add_format(ColorDepth::new(8), format_8);

        // 16-bit RGB565
        let format_16 = PixelFormat::new(
            16,
            ColorDepth::new(16),
            ColorField {
                offset: 11,
                size: 5,
            },
            ColorField { offset: 5, size: 6 },
            ColorField { offset: 0, size: 5 },
            None,
            ByteOrder::LittleEndian,
            32,
        );
        self.add_format(ColorDepth::new(16), format_16);

        // 24-bit RGB888
        let format_24 = PixelFormat::new(
            24,
            ColorDepth::new(24),
            ColorField {
                offset: 16,
                size: 8,
            },
            ColorField { offset: 8, size: 8 },
            ColorField { offset: 0, size: 8 },
            None,
            ByteOrder::LittleEndian,
            32,
        );
        self.add_format(ColorDepth::new(24), format_24);

        // 32-bit RGBA8888
        let format_32 = PixelFormat::new(
            32,
            ColorDepth::new(32),
            ColorField {
                offset: 16,
                size: 8,
            },
            ColorField { offset: 8, size: 8 },
            ColorField { offset: 0, size: 8 },
            Some(ColorField {
                offset: 24,
                size: 8,
            }),
            ByteOrder::LittleEndian,
            32,
        );
        self.add_format(ColorDepth::new(32), format_32);
    }

    /// Add a pixel format
    pub fn add_format(&mut self, depth: ColorDepth, format: PixelFormat) {
        self.formats
            .entry(depth)
            .or_insert_with(Vec::new)
            .push(format);
    }

    /// Get available formats for a depth
    pub fn get_formats(&self, depth: ColorDepth) -> Vec<&PixelFormat> {
        self.formats
            .get(&depth)
            .map(|formats| formats.iter().collect())
            .unwrap_or_default()
    }

    /// Find the best format for a depth
    pub fn find_best_format(&self, depth: ColorDepth) -> Option<&PixelFormat> {
        self.get_formats(depth).into_iter().next()
    }

    /// Check if a format is supported
    pub fn supports_format(&self, depth: ColorDepth, bits_per_pixel: u8) -> bool {
        if let Some(formats) = self.formats.get(&depth) {
            formats.iter().any(|f| f.bits_per_pixel == bits_per_pixel)
        } else {
            false
        }
    }
}

impl Default for FormatManager {
    fn default() -> Self {
        Self::new()
    }
}
