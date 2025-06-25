//! Visual selection logic
//!
//! This module provides logic for selecting appropriate visuals based on
//! various criteria such as application requirements, performance, and compatibility.

use crate::x11::VisualId;
use crate::x11::visuals::types::{VisualClass, VisualInfo};
use std::collections::HashMap;

/// Criteria for visual selection
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionCriteria {
    /// Preferred visual class
    pub preferred_class: Option<VisualClass>,
    /// Minimum color depth
    pub min_depth: Option<u8>,
    /// Maximum color depth
    pub max_depth: Option<u8>,
    /// Minimum bits per RGB component
    pub min_bits_per_rgb: Option<u8>,
    /// Maximum bits per RGB component  
    pub max_bits_per_rgb: Option<u8>,
    /// Minimum colormap entries
    pub min_colormap_entries: Option<u16>,
    /// Maximum colormap entries
    pub max_colormap_entries: Option<u16>,
    /// Require modifiable colormap
    pub require_modifiable_colormap: bool,
    /// Prefer hardware acceleration
    pub prefer_hardware_acceleration: bool,
    /// Specific visual ID to match
    pub specific_visual_id: Option<VisualId>,
    /// Screen number constraint
    pub screen: Option<u8>,
    /// Performance priority (0.0 = compatibility, 1.0 = performance)
    pub performance_priority: f32,
}

impl Default for SelectionCriteria {
    fn default() -> Self {
        Self {
            preferred_class: None,
            min_depth: None,
            max_depth: None,
            min_bits_per_rgb: None,
            max_bits_per_rgb: None,
            min_colormap_entries: None,
            max_colormap_entries: None,
            require_modifiable_colormap: false,
            prefer_hardware_acceleration: true,
            specific_visual_id: None,
            screen: None,
            performance_priority: 0.5,
        }
    }
}

impl SelectionCriteria {
    /// Create criteria for basic color applications
    pub fn basic_color() -> Self {
        Self {
            preferred_class: Some(VisualClass::TrueColor),
            min_depth: Some(15),
            min_bits_per_rgb: Some(5),
            ..Default::default()
        }
    }

    /// Create criteria for high-quality graphics applications
    pub fn high_quality_graphics() -> Self {
        Self {
            preferred_class: Some(VisualClass::TrueColor),
            min_depth: Some(24),
            min_bits_per_rgb: Some(8),
            prefer_hardware_acceleration: true,
            performance_priority: 0.8,
            ..Default::default()
        }
    }

    /// Create criteria for indexed color applications (e.g., legacy apps)
    pub fn indexed_color() -> Self {
        Self {
            preferred_class: Some(VisualClass::PseudoColor),
            min_depth: Some(8),
            max_depth: Some(8),
            require_modifiable_colormap: true,
            performance_priority: 0.3,
            ..Default::default()
        }
    }

    /// Create criteria for grayscale applications
    pub fn grayscale() -> Self {
        Self {
            preferred_class: Some(VisualClass::GrayScale),
            min_depth: Some(4),
            max_depth: Some(8),
            ..Default::default()
        }
    }

    /// Create criteria optimized for text rendering
    pub fn text_rendering() -> Self {
        Self {
            preferred_class: Some(VisualClass::PseudoColor),
            min_depth: Some(8),
            max_depth: Some(16),
            prefer_hardware_acceleration: false,
            performance_priority: 0.2,
            ..Default::default()
        }
    }

    /// Create criteria for a specific visual ID
    pub fn specific_visual(visual_id: VisualId) -> Self {
        Self {
            specific_visual_id: Some(visual_id),
            ..Default::default()
        }
    }

    /// Create criteria for a specific screen
    pub fn for_screen(screen: u8) -> Self {
        Self {
            screen: Some(screen),
            ..Default::default()
        }
    }
}

/// Result of visual selection
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// Selected visual information
    pub visual_info: VisualInfo,
    /// Quality score (0.0 = poor match, 1.0 = perfect match)
    pub quality_score: f32,
    /// Reasons for selection
    pub selection_reasons: Vec<SelectionReason>,
    /// Warnings about the selection
    pub warnings: Vec<SelectionWarning>,
}

/// Reason for visual selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionReason {
    /// Exact match for criteria
    ExactMatch,
    /// Default visual for screen
    DefaultVisual,
    /// Best available match
    BestAvailable,
    /// Hardware acceleration available
    HardwareAccelerated,
    /// Compatible with requested class
    CompatibleClass,
    /// Sufficient color depth
    SufficientDepth,
    /// Meets performance requirements
    PerformanceOptimized,
}

/// Warning about visual selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionWarning {
    /// Selected visual has lower quality than requested
    LowerQuality(String),
    /// No hardware acceleration available
    NoHardwareAcceleration,
    /// Compatibility issues possible
    CompatibilityIssues(String),
    /// Performance may be degraded
    PerformanceDegraded(String),
    /// Fallback to default visual
    FallbackToDefault,
}

/// Visual selector for choosing appropriate visuals
#[derive(Debug)]
pub struct VisualSelector {
    /// Cache of visual rankings by criteria
    selection_cache: HashMap<String, Vec<(VisualId, f32)>>,
    /// Performance characteristics of visuals
    performance_info: HashMap<VisualId, PerformanceInfo>,
}

/// Performance information for a visual
#[derive(Debug, Clone)]
struct PerformanceInfo {
    /// Hardware acceleration available
    hardware_accelerated: bool,
    /// Relative performance score
    performance_score: f32,
    /// Memory usage efficiency
    memory_efficiency: f32,
}

impl VisualSelector {
    /// Create a new visual selector
    pub fn new() -> Self {
        Self {
            selection_cache: HashMap::new(),
            performance_info: HashMap::new(),
        }
    }

    /// Select the best visual matching the given criteria
    pub fn select_visual<T: VisualSystem>(
        &mut self,
        criteria: &SelectionCriteria,
        system: &T,
    ) -> Option<SelectionResult> {
        // If a specific visual is requested, try to find it
        if let Some(visual_id) = criteria.specific_visual_id {
            if let Some(visual_info) = system.find_visual(visual_id) {
                if self.matches_criteria(&visual_info, criteria) {
                    return Some(SelectionResult {
                        visual_info,
                        quality_score: 1.0,
                        selection_reasons: vec![SelectionReason::ExactMatch],
                        warnings: Vec::new(),
                    });
                }
            }
        }

        // Get all available visuals for the screen
        let screen = criteria.screen.unwrap_or(0);
        let available_visuals = system.get_visuals(screen);

        if available_visuals.is_empty() {
            return None;
        }

        // Score and rank all visuals
        let mut scored_visuals: Vec<(
            VisualInfo,
            f32,
            Vec<SelectionReason>,
            Vec<SelectionWarning>,
        )> = available_visuals
            .into_iter()
            .filter_map(|visual_info| {
                let score = self.score_visual(&visual_info, criteria);
                if score > 0.0 {
                    let (reasons, warnings) = self.analyze_selection(&visual_info, criteria);
                    Some((visual_info, score, reasons, warnings))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        scored_visuals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return the best match
        scored_visuals.into_iter().next().map(
            |(visual_info, quality_score, selection_reasons, warnings)| SelectionResult {
                visual_info,
                quality_score,
                selection_reasons,
                warnings,
            },
        )
    }

    /// Select multiple visuals matching criteria, ranked by quality
    pub fn select_multiple_visuals<T: VisualSystem>(
        &mut self,
        criteria: &SelectionCriteria,
        system: &T,
        max_results: usize,
    ) -> Vec<SelectionResult> {
        let screen = criteria.screen.unwrap_or(0);
        let available_visuals = system.get_visuals(screen);

        let mut results: Vec<SelectionResult> = available_visuals
            .into_iter()
            .filter_map(|visual_info| {
                let score = self.score_visual(&visual_info, criteria);
                if score > 0.0 {
                    let (reasons, warnings) = self.analyze_selection(&visual_info, criteria);
                    Some(SelectionResult {
                        visual_info,
                        quality_score: score,
                        selection_reasons: reasons,
                        warnings,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by quality score (highest first)
        results.sort_by(|a, b| {
            b.quality_score
                .partial_cmp(&a.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return up to max_results
        results.into_iter().take(max_results).collect()
    }

    /// Get the default visual for a screen with fallback logic
    pub fn get_default_visual<T: VisualSystem>(
        &self,
        screen: u8,
        system: &T,
    ) -> Option<SelectionResult> {
        // Try to get the system default first
        if let Some(default_visual) = system.get_default_visual(screen) {
            return Some(SelectionResult {
                visual_info: default_visual,
                quality_score: 0.9,
                selection_reasons: vec![SelectionReason::DefaultVisual],
                warnings: Vec::new(),
            });
        }

        // Fallback: find the best TrueColor visual
        let fallback_criteria = SelectionCriteria::basic_color();
        let mut selector = VisualSelector::new();

        if let Some(mut result) = selector.select_visual(&fallback_criteria, system) {
            result
                .selection_reasons
                .push(SelectionReason::BestAvailable);
            result.warnings.push(SelectionWarning::FallbackToDefault);
            Some(result)
        } else {
            // Ultimate fallback: any visual
            let visuals = system.get_visuals(screen);
            visuals
                .into_iter()
                .next()
                .map(|visual_info| SelectionResult {
                    visual_info,
                    quality_score: 0.1,
                    selection_reasons: vec![SelectionReason::BestAvailable],
                    warnings: vec![SelectionWarning::FallbackToDefault],
                })
        }
    }

    /// Check if a visual matches the given criteria
    fn matches_criteria(&self, visual_info: &VisualInfo, criteria: &SelectionCriteria) -> bool {
        // Check screen
        if let Some(screen) = criteria.screen {
            if visual_info.screen != screen {
                return false;
            }
        }

        // Check visual class
        if let Some(preferred_class) = criteria.preferred_class {
            if visual_info.visual.class != preferred_class {
                return false;
            }
        }

        // Check depth constraints
        if let Some(min_depth) = criteria.min_depth {
            if visual_info.depth < min_depth {
                return false;
            }
        }

        if let Some(max_depth) = criteria.max_depth {
            if visual_info.depth > max_depth {
                return false;
            }
        }

        // Check bits per RGB constraints
        if let Some(min_bits) = criteria.min_bits_per_rgb {
            if visual_info.visual.bits_per_rgb < min_bits {
                return false;
            }
        }

        if let Some(max_bits) = criteria.max_bits_per_rgb {
            if visual_info.visual.bits_per_rgb > max_bits {
                return false;
            }
        }

        // Check colormap entries constraints
        if let Some(min_entries) = criteria.min_colormap_entries {
            if visual_info.visual.colormap_entries < min_entries {
                return false;
            }
        }

        if let Some(max_entries) = criteria.max_colormap_entries {
            if visual_info.visual.colormap_entries > max_entries {
                return false;
            }
        }

        // Check modifiable colormap requirement
        if criteria.require_modifiable_colormap {
            if !matches!(
                visual_info.visual.class,
                VisualClass::GrayScale | VisualClass::PseudoColor | VisualClass::DirectColor
            ) {
                return false;
            }
        }

        true
    }

    /// Score a visual against the criteria (0.0 = no match, 1.0 = perfect match)
    fn score_visual(&self, visual_info: &VisualInfo, criteria: &SelectionCriteria) -> f32 {
        if !self.matches_criteria(visual_info, criteria) {
            return 0.0;
        }

        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Visual class match (weight: 30%)
        let class_weight = 0.3;
        if let Some(preferred_class) = criteria.preferred_class {
            if visual_info.visual.class == preferred_class {
                score += class_weight;
            } else if self.is_compatible_class(visual_info.visual.class, preferred_class) {
                score += class_weight * 0.7;
            } else {
                score += class_weight * 0.3;
            }
        } else {
            score += class_weight * 0.8; // Good default
        }
        total_weight += class_weight;

        // Depth score (weight: 25%)
        let depth_weight = 0.25;
        score += depth_weight * self.score_depth(visual_info.depth, criteria);
        total_weight += depth_weight;

        // Bits per RGB score (weight: 20%)
        let bits_weight = 0.2;
        score += bits_weight * self.score_bits_per_rgb(visual_info.visual.bits_per_rgb, criteria);
        total_weight += bits_weight;

        // Performance score (weight: 15%)
        let perf_weight = 0.15;
        score += perf_weight * self.score_performance(visual_info.visual.visual_id, criteria);
        total_weight += perf_weight;

        // Colormap score (weight: 10%)
        let colormap_weight = 0.1;
        score += colormap_weight * self.score_colormap(visual_info, criteria);
        total_weight += colormap_weight;

        score / total_weight
    }

    /// Check if two visual classes are compatible
    fn is_compatible_class(&self, actual: VisualClass, preferred: VisualClass) -> bool {
        match (actual, preferred) {
            // Direct color classes are compatible
            (VisualClass::TrueColor, VisualClass::DirectColor)
            | (VisualClass::DirectColor, VisualClass::TrueColor) => true,

            // Gray classes are compatible
            (VisualClass::StaticGray, VisualClass::GrayScale)
            | (VisualClass::GrayScale, VisualClass::StaticGray) => true,

            // Color classes are somewhat compatible
            (VisualClass::StaticColor, VisualClass::PseudoColor)
            | (VisualClass::PseudoColor, VisualClass::StaticColor) => true,

            _ => false,
        }
    }

    /// Score depth against criteria
    fn score_depth(&self, depth: u8, criteria: &SelectionCriteria) -> f32 {
        let min_depth = criteria.min_depth.unwrap_or(1);
        let max_depth = criteria.max_depth.unwrap_or(32);

        if depth < min_depth || depth > max_depth {
            return 0.0;
        }

        // Prefer common depths
        match depth {
            24 | 32 => 1.0, // Ideal for most applications
            16 => 0.9,      // Good for performance
            15 => 0.85,     // Decent color
            8 => 0.7,       // Basic color
            4 => 0.5,       // Limited color
            1 => 0.3,       // Monochrome
            _ => 0.6,       // Unusual depth
        }
    }

    /// Score bits per RGB against criteria
    fn score_bits_per_rgb(&self, bits_per_rgb: u8, criteria: &SelectionCriteria) -> f32 {
        let min_bits = criteria.min_bits_per_rgb.unwrap_or(1);
        let max_bits = criteria.max_bits_per_rgb.unwrap_or(16);

        if bits_per_rgb < min_bits || bits_per_rgb > max_bits {
            return 0.0;
        }

        // Prefer 8 bits per RGB for most applications
        match bits_per_rgb {
            8 => 1.0,            // Perfect for most uses
            6 | 7 => 0.9,        // Good quality
            5 => 0.8,            // Acceptable
            4 => 0.6,            // Limited
            10 | 12 | 16 => 0.7, // High quality but less common
            _ => 0.5,            // Unusual
        }
    }

    /// Score performance characteristics
    fn score_performance(&self, visual_id: VisualId, criteria: &SelectionCriteria) -> f32 {
        if let Some(perf_info) = self.performance_info.get(&visual_id) {
            let mut score = perf_info.performance_score;

            if criteria.prefer_hardware_acceleration && perf_info.hardware_accelerated {
                score += 0.2;
            }

            score.min(1.0)
        } else {
            // Default performance score
            if criteria.prefer_hardware_acceleration {
                0.5 // Unknown, assume moderate performance
            } else {
                0.8 // Don't care about performance
            }
        }
    }

    /// Score colormap characteristics
    fn score_colormap(&self, visual_info: &VisualInfo, criteria: &SelectionCriteria) -> f32 {
        let entries = visual_info.visual.colormap_entries;

        // Check if modifiable colormap is required
        if criteria.require_modifiable_colormap {
            let is_modifiable = matches!(
                visual_info.visual.class,
                VisualClass::GrayScale | VisualClass::PseudoColor | VisualClass::DirectColor
            );
            if !is_modifiable {
                return 0.0;
            }
        }

        // Score based on colormap size
        match entries {
            256 => 1.0,                // Standard size
            128 | 512 => 0.9,          // Reasonable sizes
            64 | 1024 => 0.8,          // Usable sizes
            _ if entries >= 16 => 0.6, // Minimal color
            _ => 0.3,                  // Very limited
        }
    }

    /// Analyze the reasons and warnings for a selection
    fn analyze_selection(
        &self,
        visual_info: &VisualInfo,
        criteria: &SelectionCriteria,
    ) -> (Vec<SelectionReason>, Vec<SelectionWarning>) {
        let mut reasons = Vec::new();
        let mut warnings = Vec::new();

        // Check for exact matches
        if let Some(preferred_class) = criteria.preferred_class {
            if visual_info.visual.class == preferred_class {
                reasons.push(SelectionReason::ExactMatch);
            } else if self.is_compatible_class(visual_info.visual.class, preferred_class) {
                reasons.push(SelectionReason::CompatibleClass);
                warnings.push(SelectionWarning::LowerQuality(format!(
                    "Visual class {:?} instead of preferred {:?}",
                    visual_info.visual.class, preferred_class
                )));
            }
        }

        // Check depth adequacy
        if let Some(min_depth) = criteria.min_depth {
            if visual_info.depth >= min_depth {
                reasons.push(SelectionReason::SufficientDepth);
            }
        }

        // Check performance
        if criteria.prefer_hardware_acceleration {
            if let Some(perf_info) = self.performance_info.get(&visual_info.visual.visual_id) {
                if perf_info.hardware_accelerated {
                    reasons.push(SelectionReason::HardwareAccelerated);
                } else {
                    warnings.push(SelectionWarning::NoHardwareAcceleration);
                }
            } else {
                warnings.push(SelectionWarning::NoHardwareAcceleration);
            }
        }

        // Performance warnings
        if criteria.performance_priority > 0.7 {
            reasons.push(SelectionReason::PerformanceOptimized);
        }

        (reasons, warnings)
    }

    /// Update performance information for a visual
    pub fn update_performance_info(
        &mut self,
        visual_id: VisualId,
        hardware_accelerated: bool,
        performance_score: f32,
    ) {
        self.performance_info.insert(
            visual_id,
            PerformanceInfo {
                hardware_accelerated,
                performance_score,
                memory_efficiency: 0.8, // Default value
            },
        );
    }

    /// Clear the selection cache
    pub fn clear_cache(&mut self) {
        self.selection_cache.clear();
    }
}

impl Default for VisualSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for visual system access (to avoid circular dependencies)
pub trait VisualSystem {
    /// Get all available visuals for a screen
    fn get_visuals(&self, screen: u8) -> Vec<VisualInfo>;

    /// Get the default visual for a screen
    fn get_default_visual(&self, screen: u8) -> Option<VisualInfo>;

    /// Find a visual by ID
    fn find_visual(&self, visual_id: VisualId) -> Option<VisualInfo>;
}
