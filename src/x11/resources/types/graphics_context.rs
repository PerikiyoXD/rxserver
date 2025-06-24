//! Graphics Context resource implementation
//!
//! Graphics contexts (GCs) define how drawing operations are performed,
//! including colors, line styles, fill patterns, and other drawing attributes.

use crate::x11::protocol::types::{ClientId, XID};
use crate::x11::resources::{LifecycleError, Resource, ResourceType};

/// Graphics Context function modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GCFunction {
    Clear = 0,
    And = 1,
    AndReverse = 2,
    Copy = 3,
    AndInverted = 4,
    NoOp = 5,
    Xor = 6,
    Or = 7,
    Nor = 8,
    Equiv = 9,
    Invert = 10,
    OrReverse = 11,
    CopyInverted = 12,
    OrInverted = 13,
    Nand = 14,
    Set = 15,
}

/// Line styles for drawing operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid = 0,
    OnOffDash = 1,
    DoubleDash = 2,
}

/// Cap styles for line endings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapStyle {
    NotLast = 0,
    Butt = 1,
    Round = 2,
    Projecting = 3,
}

/// Join styles for line connections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinStyle {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

/// Fill styles for area filling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillStyle {
    Solid = 0,
    Tiled = 1,
    Stippled = 2,
    OpaqueStippled = 3,
}

/// Fill rules for polygon filling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    EvenOdd = 0,
    Winding = 1,
}

/// Subwindow modes for clipping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubwindowMode {
    ClipByChildren = 0,
    IncludeInferiors = 1,
}

/// Arc modes for arc drawing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcMode {
    Chord = 0,
    PieSlice = 1,
}

/// Graphics Context attributes
#[derive(Debug, Clone)]
pub struct GCAttributes {
    /// Function for logical operations
    pub function: GCFunction,
    /// Plane mask for bit operations
    pub plane_mask: u32,
    /// Foreground pixel value
    pub foreground: u32,
    /// Background pixel value
    pub background: u32,
    /// Line width in pixels
    pub line_width: u16,
    /// Line style
    pub line_style: LineStyle,
    /// Cap style for line endings
    pub cap_style: CapStyle,
    /// Join style for line connections
    pub join_style: JoinStyle,
    /// Fill style for areas
    pub fill_style: FillStyle,
    /// Fill rule for polygons
    pub fill_rule: FillRule,
    /// Tile pixmap XID
    pub tile: Option<XID>,
    /// Stipple pixmap XID
    pub stipple: Option<XID>,
    /// Tile/stipple X origin
    pub tile_stipple_x_origin: i16,
    /// Tile/stipple Y origin
    pub tile_stipple_y_origin: i16,
    /// Font XID for text operations
    pub font: Option<XID>,
    /// Subwindow mode for clipping
    pub subwindow_mode: SubwindowMode,
    /// Graphics exposures flag
    pub graphics_exposures: bool,
    /// Clip X origin
    pub clip_x_origin: i16,
    /// Clip Y origin
    pub clip_y_origin: i16,
    /// Clip mask pixmap XID
    pub clip_mask: Option<XID>,
    /// Dash offset for dashed lines
    pub dash_offset: u16,
    /// Dash list for line patterns
    pub dashes: Vec<u8>,
    /// Arc mode for arc drawing
    pub arc_mode: ArcMode,
}

impl Default for GCAttributes {
    fn default() -> Self {
        Self {
            function: GCFunction::Copy,
            plane_mask: u32::MAX,
            foreground: 0,
            background: 1,
            line_width: 0,
            line_style: LineStyle::Solid,
            cap_style: CapStyle::Butt,
            join_style: JoinStyle::Miter,
            fill_style: FillStyle::Solid,
            fill_rule: FillRule::EvenOdd,
            tile: None,
            stipple: None,
            tile_stipple_x_origin: 0,
            tile_stipple_y_origin: 0,
            font: None,
            subwindow_mode: SubwindowMode::ClipByChildren,
            graphics_exposures: true,
            clip_x_origin: 0,
            clip_y_origin: 0,
            clip_mask: None,
            dash_offset: 0,
            dashes: vec![4, 4], // Default dash pattern
            arc_mode: ArcMode::PieSlice,
        }
    }
}

/// Graphics Context resource implementation
#[derive(Debug)]
pub struct GraphicsContextResource {
    /// Unique identifier for this GC
    xid: XID,
    /// Client that owns this GC
    owner: ClientId,
    /// GC attributes
    attributes: GCAttributes,
    /// Drawable this GC is associated with
    drawable: XID,
    /// Cache of computed values for performance
    cache: GCCache,
    /// Validation state
    is_valid: bool,
    /// Dependencies on other resources
    dependencies: Vec<XID>,
}

/// Cache for computed GC values to improve performance
#[derive(Debug, Default)]
struct GCCache {
    /// Cached effective foreground color
    effective_foreground: Option<u32>,
    /// Cached effective background color
    effective_background: Option<u32>,
    /// Cache validity flags
    colors_valid: bool,
    patterns_valid: bool,
}

impl GraphicsContextResource {
    /// Create a new graphics context resource
    pub fn new(
        xid: XID,
        owner: ClientId,
        drawable: XID,
        attributes: Option<GCAttributes>,
    ) -> Result<Self, LifecycleError> {
        let mut gc = Self {
            xid,
            owner,
            attributes: attributes.unwrap_or_default(),
            drawable,
            cache: GCCache::default(),
            is_valid: true,
            dependencies: Vec::new(),
        };

        // Validate the initial configuration
        gc.validate_attributes()?;
        gc.update_dependencies();
        gc.invalidate_cache();

        Ok(gc)
    }

    /// Get the drawable this GC is associated with
    pub fn drawable(&self) -> XID {
        self.drawable
    }

    /// Get GC attributes
    pub fn attributes(&self) -> &GCAttributes {
        &self.attributes
    }

    /// Update GC attributes
    pub fn update_attributes(&mut self, updates: GCAttributeUpdates) -> Result<(), LifecycleError> {
        // Apply updates
        if let Some(function) = updates.function {
            self.attributes.function = function;
        }
        if let Some(plane_mask) = updates.plane_mask {
            self.attributes.plane_mask = plane_mask;
        }
        if let Some(foreground) = updates.foreground {
            self.attributes.foreground = foreground;
        }
        if let Some(background) = updates.background {
            self.attributes.background = background;
        }
        if let Some(line_width) = updates.line_width {
            self.attributes.line_width = line_width;
        }
        if let Some(line_style) = updates.line_style {
            self.attributes.line_style = line_style;
        }
        if let Some(cap_style) = updates.cap_style {
            self.attributes.cap_style = cap_style;
        }
        if let Some(join_style) = updates.join_style {
            self.attributes.join_style = join_style;
        }
        if let Some(fill_style) = updates.fill_style {
            self.attributes.fill_style = fill_style;
        }
        if let Some(fill_rule) = updates.fill_rule {
            self.attributes.fill_rule = fill_rule;
        }
        if let Some(tile) = updates.tile {
            self.attributes.tile = tile;
        }
        if let Some(stipple) = updates.stipple {
            self.attributes.stipple = stipple;
        }
        if let Some(font) = updates.font {
            self.attributes.font = font;
        }
        if let Some(clip_mask) = updates.clip_mask {
            self.attributes.clip_mask = clip_mask;
        }
        if let Some(dashes) = updates.dashes {
            self.attributes.dashes = dashes;
        }

        // Validate the updated configuration
        self.validate_attributes()?;
        self.update_dependencies();
        self.invalidate_cache();

        Ok(())
    }

    /// Check if the GC is valid for drawing operations
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Invalidate the GC (usually due to drawable destruction)
    pub fn invalidate(&mut self) {
        self.is_valid = false;
        self.invalidate_cache();
    }

    /// Get effective foreground color (with caching)
    pub fn effective_foreground(&mut self) -> u32 {
        if !self.cache.colors_valid {
            self.cache.effective_foreground = Some(self.compute_effective_foreground());
            self.cache.colors_valid = true;
        }
        self.cache
            .effective_foreground
            .unwrap_or(self.attributes.foreground)
    }

    /// Get effective background color (with caching)
    pub fn effective_background(&mut self) -> u32 {
        if !self.cache.colors_valid {
            self.cache.effective_background = Some(self.compute_effective_background());
            self.cache.colors_valid = true;
        }
        self.cache
            .effective_background
            .unwrap_or(self.attributes.background)
    }

    /// Validate GC attributes
    fn validate_attributes(&self) -> Result<(), LifecycleError> {
        // Validate line width
        if self.attributes.line_width > 32767 {
            return Err(LifecycleError::InitializationFailed(
                "Line width exceeds maximum value".into(),
            ));
        }

        // Validate dash pattern
        if self.attributes.dashes.is_empty() {
            return Err(LifecycleError::InitializationFailed(
                "Dash pattern cannot be empty".into(),
            ));
        }

        // Validate dash values
        for &dash in &self.attributes.dashes {
            if dash == 0 {
                return Err(LifecycleError::InitializationFailed(
                    "Dash values must be non-zero".into(),
                ));
            }
        }

        Ok(())
    }

    /// Update dependencies based on current attributes
    fn update_dependencies(&mut self) {
        self.dependencies.clear();

        // Add drawable dependency
        self.dependencies.push(self.drawable);

        // Add resource dependencies
        if let Some(tile) = self.attributes.tile {
            self.dependencies.push(tile);
        }
        if let Some(stipple) = self.attributes.stipple {
            self.dependencies.push(stipple);
        }
        if let Some(font) = self.attributes.font {
            self.dependencies.push(font);
        }
        if let Some(clip_mask) = self.attributes.clip_mask {
            self.dependencies.push(clip_mask);
        }
    }

    /// Invalidate cached values
    fn invalidate_cache(&mut self) {
        self.cache.colors_valid = false;
        self.cache.patterns_valid = false;
        self.cache.effective_foreground = None;
        self.cache.effective_background = None;
    }

    /// Compute effective foreground color
    fn compute_effective_foreground(&self) -> u32 {
        // Apply plane mask to foreground
        self.attributes.foreground & self.attributes.plane_mask
    }

    /// Compute effective background color
    fn compute_effective_background(&self) -> u32 {
        // Apply plane mask to background
        self.attributes.background & self.attributes.plane_mask
    }
}

/// Structure for updating GC attributes
#[derive(Debug, Default)]
pub struct GCAttributeUpdates {
    pub function: Option<GCFunction>,
    pub plane_mask: Option<u32>,
    pub foreground: Option<u32>,
    pub background: Option<u32>,
    pub line_width: Option<u16>,
    pub line_style: Option<LineStyle>,
    pub cap_style: Option<CapStyle>,
    pub join_style: Option<JoinStyle>,
    pub fill_style: Option<FillStyle>,
    pub fill_rule: Option<FillRule>,
    pub tile: Option<Option<XID>>,
    pub stipple: Option<Option<XID>>,
    pub font: Option<Option<XID>>,
    pub clip_mask: Option<Option<XID>>,
    pub dashes: Option<Vec<u8>>,
}

impl Resource for GraphicsContextResource {
    fn resource_type(&self) -> ResourceType {
        ResourceType::GraphicsContext
    }

    fn xid(&self) -> XID {
        self.xid
    }

    fn owner(&self) -> ClientId {
        self.owner
    }

    fn prepare_destroy(&mut self) -> Result<(), LifecycleError> {
        // Invalidate the GC
        self.invalidate();

        // Clear dependencies
        self.dependencies.clear();

        Ok(())
    }

    fn dependencies(&self) -> Vec<XID> {
        self.dependencies.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_creation() {
        let gc = GraphicsContextResource::new(100, 1, 50, None).unwrap();
        assert_eq!(gc.xid(), 100);
        assert_eq!(gc.owner(), 1);
        assert_eq!(gc.drawable(), 50);
        assert_eq!(gc.resource_type(), ResourceType::GraphicsContext);
        assert!(gc.is_valid());
    }

    #[test]
    fn test_gc_attributes_update() {
        let mut gc = GraphicsContextResource::new(100, 1, 50, None).unwrap();

        let mut updates = GCAttributeUpdates::default();
        updates.foreground = Some(0xFF0000);
        updates.line_width = Some(5);

        gc.update_attributes(updates).unwrap();
        assert_eq!(gc.attributes().foreground, 0xFF0000);
        assert_eq!(gc.attributes().line_width, 5);
    }

    #[test]
    fn test_gc_dependencies() {
        let mut attributes = GCAttributes::default();
        attributes.tile = Some(200);
        attributes.font = Some(300);

        let gc = GraphicsContextResource::new(100, 1, 50, Some(attributes)).unwrap();

        let deps = gc.dependencies();
        assert!(deps.contains(&50)); // drawable
        assert!(deps.contains(&200)); // tile
        assert!(deps.contains(&300)); // font
    }

    #[test]
    fn test_gc_validation() {
        let mut attributes = GCAttributes::default();
        attributes.dashes = vec![]; // Empty dash pattern should fail

        let result = GraphicsContextResource::new(100, 1, 50, Some(attributes));
        assert!(result.is_err());
    }

    #[test]
    fn test_gc_invalidation() {
        let mut gc = GraphicsContextResource::new(100, 1, 50, None).unwrap();
        assert!(gc.is_valid());

        gc.invalidate();
        assert!(!gc.is_valid());
    }

    #[test]
    fn test_effective_colors() {
        let mut attributes = GCAttributes::default();
        attributes.foreground = 0xFFFFFF;
        attributes.background = 0x000000;
        attributes.plane_mask = 0xFF0000; // Red channel only

        let mut gc = GraphicsContextResource::new(100, 1, 50, Some(attributes)).unwrap();

        assert_eq!(gc.effective_foreground(), 0xFF0000); // Masked foreground
        assert_eq!(gc.effective_background(), 0x000000); // Masked background
    }
}
