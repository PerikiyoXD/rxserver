//! Visual compatibility validation
//!
//! This module provides validation logic for visual compatibility, ensuring
//! that visual configurations are valid and supported by the system.

use crate::x11::VisualId;
use crate::x11::visuals::types::{Visual, VisualClass, VisualInfo};

/// Result of visual validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Visual configuration is valid
    Valid,
    /// Visual configuration is invalid with reason
    Invalid(ValidationError),
    /// Visual configuration is valid but with warnings
    ValidWithWarnings(Vec<ValidationWarning>),
}

/// Validation errors for visual configurations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Visual ID not found
    VisualNotFound(VisualId),
    /// Unsupported depth for visual
    UnsupportedDepth { visual_id: VisualId, depth: u8 },
    /// Invalid color mask configuration
    InvalidColorMasks { visual_id: VisualId, reason: String },
    /// Incompatible visual class for operation
    IncompatibleVisualClass {
        visual_id: VisualId,
        operation: String,
    },
    /// Colormap entries exceed limit
    TooManyColormapEntries {
        visual_id: VisualId,
        entries: u16,
        max_entries: u16,
    },
    /// Invalid bits per RGB value
    InvalidBitsPerRgb {
        visual_id: VisualId,
        bits_per_rgb: u8,
    },
    /// Visual configuration is internally inconsistent
    InconsistentConfiguration {
        visual_id: VisualId,
        details: String,
    },
}

/// Validation warnings for visual configurations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationWarning {
    /// Performance may be degraded
    PerformanceWarning(String),
    /// Compatibility issue with legacy applications
    CompatibilityWarning(String),
    /// Non-standard configuration
    NonStandardConfiguration(String),
    /// Limited hardware support
    LimitedHardwareSupport(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::VisualNotFound(visual_id) => {
                write!(f, "Visual {} not found", visual_id)
            }
            ValidationError::UnsupportedDepth { visual_id, depth } => {
                write!(f, "Visual {} does not support depth {}", visual_id, depth)
            }
            ValidationError::InvalidColorMasks { visual_id, reason } => {
                write!(
                    f,
                    "Visual {} has invalid color masks: {}",
                    visual_id, reason
                )
            }
            ValidationError::IncompatibleVisualClass {
                visual_id,
                operation,
            } => {
                write!(
                    f,
                    "Visual {} class incompatible with operation {}",
                    visual_id, operation
                )
            }
            ValidationError::TooManyColormapEntries {
                visual_id,
                entries,
                max_entries,
            } => {
                write!(
                    f,
                    "Visual {} has {} colormap entries, exceeding maximum of {}",
                    visual_id, entries, max_entries
                )
            }
            ValidationError::InvalidBitsPerRgb {
                visual_id,
                bits_per_rgb,
            } => {
                write!(
                    f,
                    "Visual {} has invalid bits per RGB: {}",
                    visual_id, bits_per_rgb
                )
            }
            ValidationError::InconsistentConfiguration { visual_id, details } => {
                write!(
                    f,
                    "Visual {} has inconsistent configuration: {}",
                    visual_id, details
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Visual validator for checking visual compatibility and validity
#[derive(Debug, Default)]
pub struct VisualValidator {
    /// Maximum supported colormap entries
    max_colormap_entries: u16,
    /// Maximum supported bits per RGB
    max_bits_per_rgb: u8,
    /// Supported depths
    supported_depths: Vec<u8>,
}

impl VisualValidator {
    /// Create a new visual validator with default limits
    pub fn new() -> Self {
        Self {
            max_colormap_entries: u16::MAX,
            max_bits_per_rgb: 16,
            supported_depths: vec![1, 4, 8, 15, 16, 24, 32],
        }
    }

    /// Create a visual validator with custom limits
    pub fn with_limits(
        max_colormap_entries: u16,
        max_bits_per_rgb: u8,
        supported_depths: Vec<u8>,
    ) -> Self {
        Self {
            max_colormap_entries,
            max_bits_per_rgb,
            supported_depths,
        }
    }

    /// Validate a visual configuration
    pub fn validate<T: VisualSystem>(
        &self,
        visual_id: VisualId,
        depth: u8,
        system: &T,
    ) -> ValidationResult {
        // Find the visual
        let visual_info = match system.find_visual(visual_id) {
            Some(info) => info,
            None => return ValidationResult::Invalid(ValidationError::VisualNotFound(visual_id)),
        };

        let mut warnings = Vec::new();

        // Check depth support
        if visual_info.depth != depth {
            return ValidationResult::Invalid(ValidationError::UnsupportedDepth {
                visual_id,
                depth,
            });
        }

        // Check if depth is supported by the system
        if !self.supported_depths.contains(&depth) {
            return ValidationResult::Invalid(ValidationError::UnsupportedDepth {
                visual_id,
                depth,
            });
        }

        // Validate the visual itself
        if let Some(error) = self.validate_visual_configuration(&visual_info.visual) {
            return ValidationResult::Invalid(error);
        }

        // Check for warnings
        warnings.extend(self.check_warnings(&visual_info.visual));

        if warnings.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::ValidWithWarnings(warnings)
        }
    }

    /// Validate visual configuration for internal consistency
    pub fn validate_visual_configuration(&self, visual: &Visual) -> Option<ValidationError> {
        // Check bits per RGB
        if visual.bits_per_rgb == 0 || visual.bits_per_rgb > self.max_bits_per_rgb {
            return Some(ValidationError::InvalidBitsPerRgb {
                visual_id: visual.visual_id,
                bits_per_rgb: visual.bits_per_rgb,
            });
        }

        // Check colormap entries
        if visual.colormap_entries > self.max_colormap_entries {
            return Some(ValidationError::TooManyColormapEntries {
                visual_id: visual.visual_id,
                entries: visual.colormap_entries,
                max_entries: self.max_colormap_entries,
            });
        }

        // Validate color masks based on visual class
        match visual.class {
            VisualClass::TrueColor | VisualClass::DirectColor => {
                if let Some(error) = self.validate_direct_color_masks(visual) {
                    return Some(error);
                }
            }
            VisualClass::StaticGray | VisualClass::GrayScale => {
                // Grayscale visuals shouldn't have color masks
                if visual.red_mask != 0 || visual.green_mask != 0 || visual.blue_mask != 0 {
                    return Some(ValidationError::InvalidColorMasks {
                        visual_id: visual.visual_id,
                        reason: "Grayscale visuals should not have color masks".to_string(),
                    });
                }
            }
            VisualClass::StaticColor | VisualClass::PseudoColor => {
                // Indexed color visuals typically don't use color masks
                if visual.red_mask != 0 || visual.green_mask != 0 || visual.blue_mask != 0 {
                    return Some(ValidationError::InvalidColorMasks {
                        visual_id: visual.visual_id,
                        reason: "Indexed color visuals typically should not have color masks"
                            .to_string(),
                    });
                }
            }
        }

        None
    }

    /// Validate color masks for direct color visuals
    fn validate_direct_color_masks(&self, visual: &Visual) -> Option<ValidationError> {
        let visual_id = visual.visual_id;

        // Check that masks are non-zero for direct color
        if visual.red_mask == 0 && visual.green_mask == 0 && visual.blue_mask == 0 {
            return Some(ValidationError::InvalidColorMasks {
                visual_id,
                reason: "Direct color visuals must have at least one non-zero color mask"
                    .to_string(),
            });
        }

        // Check for overlapping masks
        if (visual.red_mask & visual.green_mask) != 0
            || (visual.red_mask & visual.blue_mask) != 0
            || (visual.green_mask & visual.blue_mask) != 0
        {
            return Some(ValidationError::InvalidColorMasks {
                visual_id,
                reason: "Color masks must not overlap".to_string(),
            });
        }

        // Check that masks are contiguous
        for &mask in &[visual.red_mask, visual.green_mask, visual.blue_mask] {
            if mask != 0 && !self.is_contiguous_mask(mask) {
                return Some(ValidationError::InvalidColorMasks {
                    visual_id,
                    reason: "Color masks must be contiguous".to_string(),
                });
            }
        }

        None
    }

    /// Check if a mask consists of contiguous bits
    fn is_contiguous_mask(&self, mask: u32) -> bool {
        if mask == 0 {
            return true;
        }

        // Find the rightmost set bit
        let trailing_zeros = mask.trailing_zeros();
        // Shift right to align the mask
        let shifted = mask >> trailing_zeros;
        // Check if the result is all 1s (contiguous)
        (shifted & (shifted + 1)) == 0
    }

    /// Check for validation warnings
    fn check_warnings(&self, visual: &Visual) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        // Check for non-standard configurations
        match visual.class {
            VisualClass::PseudoColor if visual.bits_per_rgb > 8 => {
                warnings.push(ValidationWarning::NonStandardConfiguration(
                    "PseudoColor with more than 8 bits per RGB is unusual".to_string(),
                ));
            }
            VisualClass::StaticGray | VisualClass::GrayScale if visual.bits_per_rgb > 8 => {
                warnings.push(ValidationWarning::PerformanceWarning(
                    "High-depth grayscale may have limited hardware acceleration".to_string(),
                ));
            }
            VisualClass::DirectColor => {
                warnings.push(ValidationWarning::CompatibilityWarning(
                    "DirectColor visuals may not be supported by all applications".to_string(),
                ));
            }
            _ => {}
        }

        // Check colormap size
        if visual.colormap_entries > 4096 {
            warnings.push(ValidationWarning::PerformanceWarning(
                "Large colormaps may impact performance".to_string(),
            ));
        }

        // Check for unusual bit depths
        if visual.bits_per_rgb > 8 && !matches!(visual.class, VisualClass::TrueColor) {
            warnings.push(ValidationWarning::LimitedHardwareSupport(
                "High bit depths may have limited hardware support for this visual class"
                    .to_string(),
            ));
        }

        warnings
    }

    /// Validate visual compatibility for a specific operation
    pub fn validate_for_operation(
        &self,
        visual_id: VisualId,
        operation: &str,
        visual: &Visual,
    ) -> ValidationResult {
        match operation {
            "create_pixmap" => {
                // Pixmaps can be created for any visual
                ValidationResult::Valid
            }
            "create_window" => {
                // Windows can be created for any visual
                ValidationResult::Valid
            }
            "create_gc" => {
                // Graphics contexts can be created for any visual
                ValidationResult::Valid
            }
            "draw_text" => {
                // Text drawing works better with indexed color for font caching
                if matches!(
                    visual.class,
                    VisualClass::TrueColor | VisualClass::DirectColor
                ) {
                    ValidationResult::ValidWithWarnings(vec![
                        ValidationWarning::PerformanceWarning(
                            "Text rendering may be slower on direct color visuals".to_string(),
                        ),
                    ])
                } else {
                    ValidationResult::Valid
                }
            }
            "colormap_operations" => {
                if matches!(
                    visual.class,
                    VisualClass::GrayScale | VisualClass::PseudoColor | VisualClass::DirectColor
                ) {
                    ValidationResult::Valid
                } else {
                    ValidationResult::Invalid(ValidationError::IncompatibleVisualClass {
                        visual_id,
                        operation: operation.to_string(),
                    })
                }
            }
            _ => ValidationResult::Valid,
        }
    }

    /// Check if two visuals are compatible for operations between them
    pub fn check_compatibility(&self, visual1: &Visual, visual2: &Visual) -> ValidationResult {
        let mut warnings = Vec::new();

        // Same visual class is always compatible
        if visual1.class == visual2.class && visual1.bits_per_rgb == visual2.bits_per_rgb {
            return ValidationResult::Valid;
        }

        // Check for potentially problematic combinations
        match (visual1.class, visual2.class) {
            // Direct color to indexed color
            (
                VisualClass::TrueColor | VisualClass::DirectColor,
                VisualClass::PseudoColor | VisualClass::StaticColor,
            )
            | (
                VisualClass::PseudoColor | VisualClass::StaticColor,
                VisualClass::TrueColor | VisualClass::DirectColor,
            ) => {
                warnings.push(ValidationWarning::PerformanceWarning(
                    "Conversion between direct and indexed color may be slow".to_string(),
                ));
            }

            // Color to grayscale
            (
                VisualClass::StaticColor
                | VisualClass::PseudoColor
                | VisualClass::TrueColor
                | VisualClass::DirectColor,
                VisualClass::StaticGray | VisualClass::GrayScale,
            )
            | (
                VisualClass::StaticGray | VisualClass::GrayScale,
                VisualClass::StaticColor
                | VisualClass::PseudoColor
                | VisualClass::TrueColor
                | VisualClass::DirectColor,
            ) => {
                warnings.push(ValidationWarning::CompatibilityWarning(
                    "Color/grayscale conversion may lose information".to_string(),
                ));
            }

            _ => {}
        }

        // Check bit depth differences
        let depth_diff = (visual1.bits_per_rgb as i16 - visual2.bits_per_rgb as i16).abs();
        if depth_diff > 2 {
            warnings.push(ValidationWarning::PerformanceWarning(
                "Large bit depth differences may cause quality loss".to_string(),
            ));
        }

        if warnings.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::ValidWithWarnings(warnings)
        }
    }
}

/// Trait for visual system access (to avoid circular dependencies)
pub trait VisualSystem {
    /// Find a visual by ID
    fn find_visual(&self, visual_id: VisualId) -> Option<VisualInfo>;
}
