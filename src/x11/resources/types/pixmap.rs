//! Pixmap resource implementation
//!
//! Pixmaps are off-screen drawable areas used for image storage and manipulation.
//! They can be used as sources for drawing operations or as backing store for windows.

use crate::x11::protocol::types::{ClientId, XID};
use crate::x11::resources::{LifecycleError, Resource, ResourceType};

/// Pixmap format information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixmapFormat {
    /// Depth in bits per pixel
    pub depth: u8,
    /// Bits per pixel
    pub bits_per_pixel: u8,
    /// Scanline pad in bits
    pub scanline_pad: u8,
}

impl PixmapFormat {
    /// Create a new pixmap format
    pub fn new(depth: u8, bits_per_pixel: u8, scanline_pad: u8) -> Self {
        Self {
            depth,
            bits_per_pixel,
            scanline_pad,
        }
    }

    /// Calculate bytes per pixel
    pub fn bytes_per_pixel(&self) -> usize {
        (self.bits_per_pixel as usize + 7) / 8
    }

    /// Calculate scanline size in bytes for given width
    pub fn scanline_bytes(&self, width: u16) -> usize {
        let bits_per_line = width as usize * self.bits_per_pixel as usize;
        let padded_bits = ((bits_per_line + self.scanline_pad as usize - 1)
            / self.scanline_pad as usize)
            * self.scanline_pad as usize;
        padded_bits / 8
    }

    /// Calculate total image size in bytes
    pub fn image_size(&self, width: u16, height: u16) -> usize {
        self.scanline_bytes(width) * height as usize
    }
}

/// Pixmap resource implementation
#[derive(Debug)]
pub struct PixmapResource {
    /// Pixmap XID
    xid: XID,
    /// Owning client
    owner: ClientId,
    /// Pixmap width in pixels
    width: u16,
    /// Pixmap height in pixels
    height: u16,
    /// Pixmap format
    format: PixmapFormat,
    /// Visual ID (for compatibility with windows)
    visual: u32,
    /// Drawable that this pixmap was created from (if any)
    source_drawable: Option<XID>,
    /// Whether this pixmap is currently in use as a backing store
    is_backing_store: bool,
    /// Creation timestamp for cache management
    created_at: std::time::Instant,
}

impl PixmapResource {
    /// Create a new pixmap resource
    pub fn new(
        xid: XID,
        owner: ClientId,
        width: u16,
        height: u16,
        depth: u8,
        visual: u32,
        source_drawable: Option<XID>,
    ) -> Result<Self, LifecycleError> {
        if width == 0 || height == 0 {
            return Err(LifecycleError::InitializationFailed(
                "Width and height must be non-zero".into(),
            ));
        }

        // Determine format based on depth
        let format = Self::format_for_depth(depth)?;

        Ok(Self {
            xid,
            owner,
            width,
            height,
            format,
            visual,
            source_drawable,
            is_backing_store: false,
            created_at: std::time::Instant::now(),
        })
    }

    /// Get pixmap width
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get pixmap height
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Get pixmap depth
    pub fn depth(&self) -> u8 {
        self.format.depth
    }

    /// Get pixmap format
    pub fn format(&self) -> PixmapFormat {
        self.format
    }

    /// Get visual ID
    pub fn visual(&self) -> u32 {
        self.visual
    }

    /// Get source drawable (if any)
    pub fn source_drawable(&self) -> Option<XID> {
        self.source_drawable
    }

    /// Check if this pixmap is being used as backing store
    pub fn is_backing_store(&self) -> bool {
        self.is_backing_store
    }

    /// Set backing store flag
    pub fn set_backing_store(&mut self, is_backing: bool) {
        self.is_backing_store = is_backing;
    }

    /// Get creation time
    pub fn created_at(&self) -> std::time::Instant {
        self.created_at
    }

    /// Get age of this pixmap
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Calculate memory usage of this pixmap
    pub fn memory_usage(&self) -> usize {
        self.format.image_size(self.width, self.height)
    }

    /// Check if dimensions are compatible with another drawable
    pub fn is_compatible_size(&self, width: u16, height: u16) -> bool {
        self.width == width && self.height == height
    }

    /// Check if format is compatible with another drawable
    pub fn is_compatible_format(&self, depth: u8, visual: u32) -> bool {
        self.format.depth == depth && self.visual == visual
    }

    /// Determine format for a given depth
    fn format_for_depth(depth: u8) -> Result<PixmapFormat, LifecycleError> {
        match depth {
            1 => Ok(PixmapFormat::new(1, 1, 32)),
            4 => Ok(PixmapFormat::new(4, 4, 32)),
            8 => Ok(PixmapFormat::new(8, 8, 32)),
            15 => Ok(PixmapFormat::new(15, 16, 32)),
            16 => Ok(PixmapFormat::new(16, 16, 32)),
            24 => Ok(PixmapFormat::new(24, 32, 32)),
            32 => Ok(PixmapFormat::new(32, 32, 32)),
            _ => Err(LifecycleError::InitializationFailed(format!(
                "Unsupported depth: {}",
                depth
            ))),
        }
    }
}

impl Resource for PixmapResource {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Pixmap
    }

    fn xid(&self) -> XID {
        self.xid
    }

    fn owner(&self) -> ClientId {
        self.owner
    }

    fn prepare_destroy(&mut self) -> Result<(), LifecycleError> {
        // Mark as no longer backing store
        self.is_backing_store = false;
        Ok(())
    }

    fn dependencies(&self) -> Vec<XID> {
        // If created from a drawable, that's a dependency
        self.source_drawable.into_iter().collect()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Pixmap statistics for resource management
#[derive(Debug, Default, Clone)]
pub struct PixmapStats {
    /// Total number of pixmaps
    pub count: usize,
    /// Total memory used by pixmaps in bytes
    pub total_memory: usize,
    /// Number of pixmaps used as backing store
    pub backing_store_count: usize,
    /// Memory used by backing store pixmaps
    pub backing_store_memory: usize,
}

impl PixmapStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pixmap to the statistics
    pub fn add_pixmap(&mut self, pixmap: &PixmapResource) {
        self.count += 1;
        let memory = pixmap.memory_usage();
        self.total_memory += memory;

        if pixmap.is_backing_store() {
            self.backing_store_count += 1;
            self.backing_store_memory += memory;
        }
    }

    /// Remove a pixmap from the statistics
    pub fn remove_pixmap(&mut self, pixmap: &PixmapResource) {
        if self.count > 0 {
            self.count -= 1;
            let memory = pixmap.memory_usage();
            self.total_memory = self.total_memory.saturating_sub(memory);

            if pixmap.is_backing_store() && self.backing_store_count > 0 {
                self.backing_store_count -= 1;
                self.backing_store_memory = self.backing_store_memory.saturating_sub(memory);
            }
        }
    }

    /// Get average memory per pixmap
    pub fn average_memory(&self) -> usize {
        if self.count > 0 {
            self.total_memory / self.count
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixmap_creation() {
        let pixmap = PixmapResource::new(123, 1, 100, 200, 24, 0x21, None).unwrap();

        assert_eq!(pixmap.xid(), 123);
        assert_eq!(pixmap.owner(), 1);
        assert_eq!(pixmap.width(), 100);
        assert_eq!(pixmap.height(), 200);
        assert_eq!(pixmap.depth(), 24);
        assert_eq!(pixmap.visual(), 0x21);
        assert!(!pixmap.is_backing_store());
    }

    #[test]
    fn test_pixmap_invalid_dimensions() {
        let result = PixmapResource::new(123, 1, 0, 200, 24, 0x21, None);
        assert!(result.is_err());

        let result = PixmapResource::new(123, 1, 100, 0, 24, 0x21, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_pixmap_format() {
        let format = PixmapFormat::new(24, 32, 32);

        assert_eq!(format.bytes_per_pixel(), 4);
        assert_eq!(format.scanline_bytes(100), 400);
        assert_eq!(format.image_size(100, 200), 80000);
    }

    #[test]
    fn test_pixmap_compatibility() {
        let pixmap = PixmapResource::new(123, 1, 100, 200, 24, 0x21, None).unwrap();

        assert!(pixmap.is_compatible_size(100, 200));
        assert!(!pixmap.is_compatible_size(100, 100));

        assert!(pixmap.is_compatible_format(24, 0x21));
        assert!(!pixmap.is_compatible_format(16, 0x21));
    }

    #[test]
    fn test_pixmap_stats() {
        let mut stats = PixmapStats::new();
        let pixmap1 = PixmapResource::new(123, 1, 100, 100, 24, 0x21, None).unwrap();
        let mut pixmap2 = PixmapResource::new(456, 1, 50, 50, 24, 0x21, None).unwrap();
        pixmap2.set_backing_store(true);

        stats.add_pixmap(&pixmap1);
        stats.add_pixmap(&pixmap2);

        assert_eq!(stats.count, 2);
        assert_eq!(stats.backing_store_count, 1);
        assert!(stats.total_memory > 0);
        assert!(stats.backing_store_memory > 0);

        stats.remove_pixmap(&pixmap1);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.backing_store_count, 1);
    }
}
