//! Cursor resource implementation
//!
//! Cursors in X11 define the appearance and behavior of the mouse pointer.
//! They can be created from pixmaps or from predefined standard cursors.

use crate::x11::protocol::types::{ClientId, XID};
use crate::x11::resources::{LifecycleError, Resource, ResourceType};

/// Cursor hotspot coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotspot {
    /// X coordinate of the hotspot
    pub x: u16,
    /// Y coordinate of the hotspot
    pub y: u16,
}

impl Hotspot {
    /// Create a new hotspot
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Default hotspot (top-left corner)
    pub const ORIGIN: Hotspot = Hotspot { x: 0, y: 0 };

    /// Center hotspot for a given size
    pub fn center(width: u16, height: u16) -> Self {
        Self {
            x: width / 2,
            y: height / 2,
        }
    }
}

/// Standard cursor shapes defined by X11
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardCursor {
    Arrow = 0,
    Hand = 2,
    Watch = 4,
    Cross = 6,
    IBeam = 8,
    Forbidden = 10,
    SizeAll = 12,
    SizeNS = 14,
    SizeWE = 16,
    SizeNWSE = 18,
    SizeNESW = 20,
    Question = 22,
    Pirate = 24,
    Coffee = 26,
    Umbrella = 28,
    Exchange = 30,
}

impl StandardCursor {
    /// Get the default hotspot for a standard cursor
    pub fn default_hotspot(self) -> Hotspot {
        match self {
            StandardCursor::Arrow => Hotspot::new(1, 1),
            StandardCursor::Hand => Hotspot::new(8, 8),
            StandardCursor::Watch => Hotspot::new(8, 8),
            StandardCursor::Cross => Hotspot::new(8, 8),
            StandardCursor::IBeam => Hotspot::new(4, 8),
            StandardCursor::Forbidden => Hotspot::new(8, 8),
            StandardCursor::SizeAll => Hotspot::new(8, 8),
            StandardCursor::SizeNS => Hotspot::new(8, 8),
            StandardCursor::SizeWE => Hotspot::new(8, 8),
            StandardCursor::SizeNWSE => Hotspot::new(8, 8),
            StandardCursor::SizeNESW => Hotspot::new(8, 8),
            StandardCursor::Question => Hotspot::new(8, 8),
            StandardCursor::Pirate => Hotspot::new(8, 8),
            StandardCursor::Coffee => Hotspot::new(8, 8),
            StandardCursor::Umbrella => Hotspot::new(8, 8),
            StandardCursor::Exchange => Hotspot::new(8, 8),
        }
    }

    /// Get the default size for a standard cursor
    pub fn default_size(self) -> (u16, u16) {
        match self {
            StandardCursor::Arrow => (16, 16),
            StandardCursor::Hand => (16, 16),
            StandardCursor::Watch => (16, 16),
            StandardCursor::Cross => (16, 16),
            StandardCursor::IBeam => (8, 16),
            StandardCursor::Forbidden => (16, 16),
            StandardCursor::SizeAll => (16, 16),
            StandardCursor::SizeNS => (16, 16),
            StandardCursor::SizeWE => (16, 16),
            StandardCursor::SizeNWSE => (16, 16),
            StandardCursor::SizeNESW => (16, 16),
            StandardCursor::Question => (16, 16),
            StandardCursor::Pirate => (16, 16),
            StandardCursor::Coffee => (16, 16),
            StandardCursor::Umbrella => (16, 16),
            StandardCursor::Exchange => (16, 16),
        }
    }
}

/// Cursor creation methods
#[derive(Debug, Clone)]
pub enum CursorSource {
    /// Create from source and mask pixmaps
    FromPixmaps {
        source: XID,
        mask: Option<XID>,
        foreground_color: (u16, u16, u16), // RGB
        background_color: (u16, u16, u16), // RGB
        hotspot: Hotspot,
    },
    /// Create from a standard cursor shape
    Standard { shape: StandardCursor, scale: f32 },
    /// Create from raw bitmap data
    FromBitmap {
        width: u16,
        height: u16,
        source_data: Vec<u8>,
        mask_data: Option<Vec<u8>>,
        hotspot: Hotspot,
        foreground_color: (u16, u16, u16),
        background_color: (u16, u16, u16),
    },
}

/// Cursor animation frame
#[derive(Debug, Clone)]
pub struct CursorFrame {
    /// Frame bitmap data
    pub data: Vec<u8>,
    /// Frame width
    pub width: u16,
    /// Frame height
    pub height: u16,
    /// Frame duration in milliseconds
    pub duration: u32,
    /// Frame hotspot
    pub hotspot: Hotspot,
}

/// Cursor resource implementation
#[derive(Debug)]
pub struct CursorResource {
    /// Unique identifier for this cursor
    xid: XID,
    /// Client that owns this cursor
    owner: ClientId,
    /// Cursor creation source
    source: CursorSource,
    /// Cursor dimensions
    width: u16,
    height: u16,
    /// Cursor hotspot
    hotspot: Hotspot,
    /// Foreground color (RGB)
    foreground_color: (u16, u16, u16),
    /// Background color (RGB)
    background_color: (u16, u16, u16),
    /// Cursor bitmap data
    bitmap_data: Vec<u8>,
    /// Cursor mask data (optional)
    mask_data: Option<Vec<u8>>,
    /// Animation frames (for animated cursors)
    frames: Vec<CursorFrame>,
    /// Current animation frame
    current_frame: usize,
    /// Dependencies on other resources
    dependencies: Vec<XID>,
    /// Cursor usage statistics
    stats: CursorStats,
}

/// Cursor usage statistics
#[derive(Debug)]
struct CursorStats {
    /// Number of times this cursor has been set
    set_count: u32,
    /// Number of windows using this cursor
    active_windows: u32,
    /// Last time this cursor was used
    last_used: std::time::SystemTime,
}

impl Default for CursorStats {
    fn default() -> Self {
        Self {
            set_count: 0,
            active_windows: 0,
            last_used: std::time::SystemTime::UNIX_EPOCH,
        }
    }
}

impl CursorResource {
    /// Create a new cursor from pixmaps
    pub fn from_pixmaps(
        xid: XID,
        owner: ClientId,
        source: XID,
        mask: Option<XID>,
        foreground_color: (u16, u16, u16),
        background_color: (u16, u16, u16),
        hotspot: Hotspot,
    ) -> Result<Self, LifecycleError> {
        let cursor_source = CursorSource::FromPixmaps {
            source,
            mask,
            foreground_color,
            background_color,
            hotspot,
        };

        let mut cursor = Self {
            xid,
            owner,
            source: cursor_source,
            width: 16, // Default size, should be queried from pixmap
            height: 16,
            hotspot,
            foreground_color,
            background_color,
            bitmap_data: Vec::new(),
            mask_data: None,
            frames: Vec::new(),
            current_frame: 0,
            dependencies: Vec::new(),
            stats: CursorStats::default(),
        };

        cursor.update_dependencies();
        cursor.generate_bitmap_data()?;

        Ok(cursor)
    }

    /// Create a new cursor from a standard shape
    pub fn from_standard(
        xid: XID,
        owner: ClientId,
        shape: StandardCursor,
        scale: f32,
    ) -> Result<Self, LifecycleError> {
        if scale <= 0.0 || scale > 10.0 {
            return Err(LifecycleError::InitializationFailed(
                "Invalid cursor scale factor".into(),
            ));
        }

        let (base_width, base_height) = shape.default_size();
        let width = ((base_width as f32) * scale) as u16;
        let height = ((base_height as f32) * scale) as u16;
        let hotspot = Hotspot::new(
            ((shape.default_hotspot().x as f32) * scale) as u16,
            ((shape.default_hotspot().y as f32) * scale) as u16,
        );

        let cursor_source = CursorSource::Standard { shape, scale };

        let mut cursor = Self {
            xid,
            owner,
            source: cursor_source,
            width,
            height,
            hotspot,
            foreground_color: (0, 0, 0),             // Black
            background_color: (65535, 65535, 65535), // White
            bitmap_data: Vec::new(),
            mask_data: None,
            frames: Vec::new(),
            current_frame: 0,
            dependencies: Vec::new(),
            stats: CursorStats::default(),
        };

        cursor.generate_standard_cursor_data(shape, scale)?;

        Ok(cursor)
    }

    /// Create a new cursor from raw bitmap data
    pub fn from_bitmap(
        xid: XID,
        owner: ClientId,
        width: u16,
        height: u16,
        source_data: Vec<u8>,
        mask_data: Option<Vec<u8>>,
        hotspot: Hotspot,
        foreground_color: (u16, u16, u16),
        background_color: (u16, u16, u16),
    ) -> Result<Self, LifecycleError> {
        if width == 0 || height == 0 {
            return Err(LifecycleError::InitializationFailed(
                "Cursor dimensions must be non-zero".into(),
            ));
        }

        if width > 1024 || height > 1024 {
            return Err(LifecycleError::InitializationFailed(
                "Cursor dimensions too large".into(),
            ));
        }

        if hotspot.x >= width || hotspot.y >= height {
            return Err(LifecycleError::InitializationFailed(
                "Hotspot outside cursor bounds".into(),
            ));
        }

        let expected_size = ((width as usize) * (height as usize) + 7) / 8; // Bitmap size
        if source_data.len() != expected_size {
            return Err(LifecycleError::InitializationFailed(
                "Invalid source data size".into(),
            ));
        }

        if let Some(ref mask) = mask_data {
            if mask.len() != expected_size {
                return Err(LifecycleError::InitializationFailed(
                    "Invalid mask data size".into(),
                ));
            }
        }

        let cursor_source = CursorSource::FromBitmap {
            width,
            height,
            source_data: source_data.clone(),
            mask_data: mask_data.clone(),
            hotspot,
            foreground_color,
            background_color,
        };

        let cursor = Self {
            xid,
            owner,
            source: cursor_source,
            width,
            height,
            hotspot,
            foreground_color,
            background_color,
            bitmap_data: source_data,
            mask_data,
            frames: Vec::new(),
            current_frame: 0,
            dependencies: Vec::new(),
            stats: CursorStats::default(),
        };

        Ok(cursor)
    }

    /// Get cursor dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Get cursor hotspot
    pub fn hotspot(&self) -> Hotspot {
        self.hotspot
    }

    /// Get cursor colors
    pub fn colors(&self) -> ((u16, u16, u16), (u16, u16, u16)) {
        (self.foreground_color, self.background_color)
    }

    /// Get cursor bitmap data
    pub fn bitmap_data(&self) -> &[u8] {
        &self.bitmap_data
    }

    /// Get cursor mask data
    pub fn mask_data(&self) -> Option<&[u8]> {
        self.mask_data.as_deref()
    }

    /// Check if this is an animated cursor
    pub fn is_animated(&self) -> bool {
        !self.frames.is_empty()
    }

    /// Get the current animation frame
    pub fn current_frame(&self) -> Option<&CursorFrame> {
        self.frames.get(self.current_frame)
    }

    /// Advance to the next animation frame
    pub fn next_frame(&mut self) -> bool {
        if self.frames.is_empty() {
            return false;
        }

        self.current_frame = (self.current_frame + 1) % self.frames.len();
        true
    }

    /// Set the current animation frame
    pub fn set_frame(&mut self, frame: usize) -> bool {
        if frame < self.frames.len() {
            self.current_frame = frame;
            true
        } else {
            false
        }
    }

    /// Add an animation frame
    pub fn add_frame(&mut self, frame: CursorFrame) {
        self.frames.push(frame);
    }

    /// Record cursor usage
    pub fn record_usage(&mut self) {
        self.stats.set_count += 1;
        self.stats.last_used = std::time::SystemTime::now();
    }

    /// Get usage statistics
    pub fn usage_stats(&self) -> (u32, u32) {
        (self.stats.set_count, self.stats.active_windows)
    }

    /// Set the number of active windows using this cursor
    pub fn set_active_windows(&mut self, count: u32) {
        self.stats.active_windows = count;
    }

    /// Update dependencies based on cursor source
    fn update_dependencies(&mut self) {
        self.dependencies.clear();

        if let CursorSource::FromPixmaps { source, mask, .. } = &self.source {
            self.dependencies.push(*source);
            if let Some(mask_xid) = mask {
                self.dependencies.push(*mask_xid);
            }
        }
    }

    /// Generate bitmap data from pixmap sources
    fn generate_bitmap_data(&mut self) -> Result<(), LifecycleError> {
        match &self.source {
            CursorSource::FromPixmaps { .. } => {
                // In a real implementation, this would query the pixmap resources
                // and extract their bitmap data
                // For now, generate placeholder data
                let size = ((self.width as usize) * (self.height as usize) + 7) / 8;
                self.bitmap_data = vec![0x55; size]; // Checkerboard pattern
                if self.mask_data.is_none() {
                    self.mask_data = Some(vec![0xFF; size]); // Full mask
                }
                Ok(())
            }
            CursorSource::FromBitmap { .. } => {
                // Data is already provided
                Ok(())
            }
            CursorSource::Standard { .. } => {
                // Will be handled by generate_standard_cursor_data
                Ok(())
            }
        }
    }

    /// Generate cursor data for standard cursors
    fn generate_standard_cursor_data(
        &mut self,
        shape: StandardCursor,
        scale: f32,
    ) -> Result<(), LifecycleError> {
        let size = ((self.width as usize) * (self.height as usize) + 7) / 8;

        match shape {
            StandardCursor::Arrow => {
                self.bitmap_data = self.generate_arrow_cursor(scale);
                self.mask_data = Some(self.generate_arrow_mask(scale));
            }
            StandardCursor::Hand => {
                self.bitmap_data = self.generate_hand_cursor(scale);
                self.mask_data = Some(vec![0xFF; size]);
            }
            StandardCursor::Watch => {
                self.bitmap_data = self.generate_watch_cursor(scale);
                self.mask_data = Some(vec![0xFF; size]);
            }
            StandardCursor::Cross => {
                self.bitmap_data = self.generate_cross_cursor(scale);
                self.mask_data = Some(vec![0xFF; size]);
            }
            StandardCursor::IBeam => {
                self.bitmap_data = self.generate_ibeam_cursor(scale);
                self.mask_data = Some(vec![0xFF; size]);
            }
            _ => {
                // For other cursors, generate a simple placeholder
                self.bitmap_data = vec![0x55; size]; // Checkerboard
                self.mask_data = Some(vec![0xFF; size]);
            }
        }

        Ok(())
    }
    /// Generate arrow cursor bitmap
    fn generate_arrow_cursor(&self, _scale: f32) -> Vec<u8> {
        let size = ((self.width as usize) * (self.height as usize) + 7) / 8;
        let mut data = vec![0u8; size];

        // Simple arrow pattern (simplified for demonstration)
        let base_pattern = [
            0b10000000, 0b11000000, 0b11100000, 0b11110000, 0b11111000, 0b11100000, 0b10100000,
            0b00100000,
        ];

        for (i, &pattern) in base_pattern.iter().enumerate() {
            if i < size {
                data[i] = pattern;
            }
        }

        data
    }
    /// Generate arrow cursor mask
    fn generate_arrow_mask(&self, _scale: f32) -> Vec<u8> {
        let size = ((self.width as usize) * (self.height as usize) + 7) / 8;
        let mut data = vec![0u8; size];

        // Arrow mask (slightly larger than the arrow itself)
        let base_pattern = [
            0b11000000, 0b11100000, 0b11110000, 0b11111000, 0b11111100, 0b11111000, 0b11110000,
            0b01110000,
        ];

        for (i, &pattern) in base_pattern.iter().enumerate() {
            if i < size {
                data[i] = pattern;
            }
        }

        data
    }
    /// Generate hand cursor bitmap
    fn generate_hand_cursor(&self, _scale: f32) -> Vec<u8> {
        let size = ((self.width as usize) * (self.height as usize) + 7) / 8;
        vec![0x7E; size] // Simple hand-like pattern
    }

    /// Generate watch cursor bitmap
    fn generate_watch_cursor(&self, _scale: f32) -> Vec<u8> {
        let size = ((self.width as usize) * (self.height as usize) + 7) / 8;
        vec![0x3C; size] // Simple watch-like pattern
    }

    /// Generate cross cursor bitmap
    fn generate_cross_cursor(&self, _scale: f32) -> Vec<u8> {
        let size = ((self.width as usize) * (self.height as usize) + 7) / 8;
        vec![0x18; size] // Simple cross pattern
    }

    /// Generate I-beam cursor bitmap
    fn generate_ibeam_cursor(&self, _scale: f32) -> Vec<u8> {
        let size = ((self.width as usize) * (self.height as usize) + 7) / 8;
        vec![0x08; size] // Simple I-beam pattern
    }
}

impl Resource for CursorResource {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Cursor
    }

    fn xid(&self) -> XID {
        self.xid
    }

    fn owner(&self) -> ClientId {
        self.owner
    }

    fn prepare_destroy(&mut self) -> Result<(), LifecycleError> {
        // Clear bitmap data
        self.bitmap_data.clear();
        self.mask_data = None;
        self.frames.clear();

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
    fn test_cursor_from_standard() {
        let cursor = CursorResource::from_standard(100, 1, StandardCursor::Arrow, 1.0).unwrap();
        assert_eq!(cursor.xid(), 100);
        assert_eq!(cursor.owner(), 1);
        assert_eq!(cursor.resource_type(), ResourceType::Cursor);
        assert_eq!(cursor.dimensions(), (16, 16));
        assert!(!cursor.bitmap_data().is_empty());
    }

    #[test]
    fn test_cursor_from_bitmap() {
        let width = 16;
        let height = 16;
        let size = ((width * height) + 7) / 8;
        let bitmap_data = vec![0x55; size as usize]; // Checkerboard pattern
        let hotspot = Hotspot::new(8, 8);

        let cursor = CursorResource::from_bitmap(
            100,
            1,
            width,
            height,
            bitmap_data.clone(),
            None,
            hotspot,
            (0, 0, 0),
            (65535, 65535, 65535),
        )
        .unwrap();

        assert_eq!(cursor.dimensions(), (width, height));
        assert_eq!(cursor.hotspot(), hotspot);
        assert_eq!(cursor.bitmap_data(), &bitmap_data);
    }

    #[test]
    fn test_cursor_invalid_dimensions() {
        let result = CursorResource::from_bitmap(
            100,
            1,
            0,
            16,
            vec![],
            None,
            Hotspot::ORIGIN,
            (0, 0, 0),
            (65535, 65535, 65535),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_cursor_invalid_hotspot() {
        let width = 16;
        let height = 16;
        let size = ((width * height) + 7) / 8;
        let bitmap_data = vec![0x55; size as usize];
        let hotspot = Hotspot::new(20, 8); // Outside bounds

        let result = CursorResource::from_bitmap(
            100,
            1,
            width,
            height,
            bitmap_data,
            None,
            hotspot,
            (0, 0, 0),
            (65535, 65535, 65535),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_cursor_invalid_scale() {
        let result = CursorResource::from_standard(100, 1, StandardCursor::Arrow, 0.0);
        assert!(result.is_err());

        let result = CursorResource::from_standard(100, 1, StandardCursor::Arrow, 15.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_cursor_animation() {
        let mut cursor = CursorResource::from_standard(100, 1, StandardCursor::Arrow, 1.0).unwrap();
        assert!(!cursor.is_animated());

        let frame = CursorFrame {
            data: vec![0xFF; 32],
            width: 16,
            height: 16,
            duration: 100,
            hotspot: Hotspot::new(8, 8),
        };

        cursor.add_frame(frame);
        assert!(cursor.is_animated());
        assert!(cursor.current_frame().is_some());
    }

    #[test]
    fn test_cursor_usage_tracking() {
        let mut cursor = CursorResource::from_standard(100, 1, StandardCursor::Arrow, 1.0).unwrap();

        let (initial_count, _) = cursor.usage_stats();
        assert_eq!(initial_count, 0);

        cursor.record_usage();
        let (count, _) = cursor.usage_stats();
        assert_eq!(count, 1);

        cursor.set_active_windows(3);
        let (_, active) = cursor.usage_stats();
        assert_eq!(active, 3);
    }

    #[test]
    fn test_hotspot_helpers() {
        let hotspot = Hotspot::center(20, 16);
        assert_eq!(hotspot.x, 10);
        assert_eq!(hotspot.y, 8);

        let arrow_hotspot = StandardCursor::Arrow.default_hotspot();
        assert_eq!(arrow_hotspot.x, 1);
        assert_eq!(arrow_hotspot.y, 1);
    }
}
