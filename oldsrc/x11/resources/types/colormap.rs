//! Colormap resource implementation
//!
//! Colormaps in X11 define the mapping from pixel values to actual colors.
//! They can be static (read-only) or dynamic (allowing color changes).

use crate::x11::protocol::types::{ClientId, Color, XId};
use crate::x11::resources::{LifecycleError, Resource, ResourceType};
use crate::x11::visuals::types::VisualClass;
use std::collections::HashMap;

/// Color allocation information
#[derive(Debug, Clone)]
pub struct ColorAllocation {
    /// Pixel value
    pub pixel: u32,
    /// Color value
    pub color: Color,
    /// Client that allocated this color
    pub client: ClientId,
    /// Whether this is a read-only or read-write allocation
    pub read_only: bool,
    /// Reference count for shared allocations
    pub ref_count: u32,
}

/// Colormap resource implementation
#[derive(Debug)]
pub struct ColormapResource {
    /// Unique identifier for this colormap
    xid: XId,
    /// Client that owns this colormap
    owner: ClientId,
    /// Associated visual XId
    visual: XId,
    /// Visual class of this colormap
    visual_class: VisualClass,
    /// Number of colormap entries
    size: u32,
    /// Colormap entries (pixel -> color mapping)
    entries: HashMap<u32, Color>,
    /// Color allocations by clients
    allocations: HashMap<u32, ColorAllocation>,
    /// Next available pixel value for dynamic allocation
    next_pixel: u32,
    /// Whether this is the default colormap
    is_default: bool,
    /// Colormap attributes
    attributes: ColormapAttributes,
}

/// Colormap attributes and configuration
#[derive(Debug, Clone)]
pub struct ColormapAttributes {
    /// Whether to install this colormap automatically
    pub auto_install: bool,
    /// Color allocation policy
    pub alloc_policy: AllocationPolicy,
    /// Maximum number of allocations per client
    pub max_allocs_per_client: u32,
}

/// Color allocation policies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationPolicy {
    /// First available pixel
    FirstAvailable,
    /// Best fit allocation
    BestFit,
    /// Closest color match
    ClosestMatch,
}

impl Default for ColormapAttributes {
    fn default() -> Self {
        Self {
            auto_install: false,
            alloc_policy: AllocationPolicy::FirstAvailable,
            max_allocs_per_client: 256,
        }
    }
}

/// Color allocation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColormapError {
    /// No more colors available
    NoMoreColors,
    /// Invalid pixel value
    InvalidPixel(u32),
    /// Color not allocated
    ColorNotAllocated(u32),
    /// Access denied
    AccessDenied,
    /// Invalid color specification
    InvalidColor,
    /// Allocation limit exceeded
    AllocationLimitExceeded,
}

impl std::fmt::Display for ColormapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColormapError::NoMoreColors => write!(f, "No more colors available in colormap"),
            ColormapError::InvalidPixel(pixel) => write!(f, "Invalid pixel value: {}", pixel),
            ColormapError::ColorNotAllocated(pixel) => write!(f, "Color not allocated: {}", pixel),
            ColormapError::AccessDenied => write!(f, "Access denied to colormap operation"),
            ColormapError::InvalidColor => write!(f, "Invalid color specification"),
            ColormapError::AllocationLimitExceeded => write!(f, "Color allocation limit exceeded"),
        }
    }
}

impl std::error::Error for ColormapError {}

impl ColormapResource {
    /// Create a new colormap resource
    pub fn new(
        xid: XId,
        owner: ClientId,
        visual: XId,
        visual_class: VisualClass,
        size: u32,
    ) -> Result<Self, LifecycleError> {
        if size == 0 || size > 65536 {
            return Err(LifecycleError::InitializationFailed(
                "Invalid colormap size".into(),
            ));
        }

        let mut colormap = Self {
            xid,
            owner,
            visual,
            visual_class,
            size,
            entries: HashMap::new(),
            allocations: HashMap::new(),
            next_pixel: 0,
            is_default: false,
            attributes: ColormapAttributes::default(),
        };

        // Initialize the colormap based on visual class
        colormap.initialize_entries()?;

        Ok(colormap)
    }

    /// Create the default colormap
    pub fn new_default(
        xid: XId,
        visual: XId,
        visual_class: VisualClass,
        size: u32,
    ) -> Result<Self, LifecycleError> {
        let mut colormap = Self::new(xid, 0, visual, visual_class, size)?;
        colormap.is_default = true;
        colormap.setup_default_colors()?;
        Ok(colormap)
    }

    /// Get the visual associated with this colormap
    pub fn visual(&self) -> XId {
        self.visual
    }

    /// Get the visual class
    pub fn visual_class(&self) -> VisualClass {
        self.visual_class
    }

    /// Get the colormap size
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Check if this is the default colormap
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    /// Get colormap attributes
    pub fn attributes(&self) -> &ColormapAttributes {
        &self.attributes
    }

    /// Update colormap attributes
    pub fn update_attributes(&mut self, attributes: ColormapAttributes) {
        self.attributes = attributes;
    }

    /// Allocate a color (read-only)
    pub fn alloc_color(&mut self, color: Color, client: ClientId) -> Result<u32, ColormapError> {
        // Check client allocation limit
        let client_allocs = self
            .allocations
            .values()
            .filter(|alloc| alloc.client == client)
            .count() as u32;

        if client_allocs >= self.attributes.max_allocs_per_client {
            return Err(ColormapError::AllocationLimitExceeded);
        }

        // Try to find an existing matching color
        for (pixel, existing_color) in &self.entries {
            if *existing_color == color {
                if let Some(allocation) = self.allocations.get_mut(pixel) {
                    if allocation.read_only {
                        allocation.ref_count += 1;
                        return Ok(*pixel);
                    }
                }
            }
        }

        // Allocate a new pixel
        let pixel = self.find_available_pixel()?;

        self.entries.insert(pixel, color);
        self.allocations.insert(
            pixel,
            ColorAllocation {
                pixel,
                color,
                client,
                read_only: true,
                ref_count: 1,
            },
        );

        Ok(pixel)
    }
    /// Allocate a color (read-write)
    pub fn alloc_color_rw(&mut self, client: ClientId) -> Result<u32, ColormapError> {
        if !self.visual_class.supports_modifiable_colormap() {
            return Err(ColormapError::AccessDenied);
        }

        // Check client allocation limit
        let client_allocs = self
            .allocations
            .values()
            .filter(|alloc| alloc.client == client)
            .count() as u32;

        if client_allocs >= self.attributes.max_allocs_per_client {
            return Err(ColormapError::AllocationLimitExceeded);
        }

        let pixel = self.find_available_pixel()?;
        let default_color = Color::BLACK;

        self.entries.insert(pixel, default_color);
        self.allocations.insert(
            pixel,
            ColorAllocation {
                pixel,
                color: default_color,
                client,
                read_only: false,
                ref_count: 1,
            },
        );

        Ok(pixel)
    }

    /// Store a color in an allocated pixel
    pub fn store_color(
        &mut self,
        pixel: u32,
        color: Color,
        client: ClientId,
    ) -> Result<(), ColormapError> {
        let allocation = self
            .allocations
            .get_mut(&pixel)
            .ok_or(ColormapError::ColorNotAllocated(pixel))?;

        if allocation.client != client {
            return Err(ColormapError::AccessDenied);
        }

        if allocation.read_only {
            return Err(ColormapError::AccessDenied);
        }

        self.entries.insert(pixel, color);
        allocation.color = color;

        Ok(())
    }

    /// Free a color allocation
    pub fn free_color(&mut self, pixel: u32, client: ClientId) -> Result<(), ColormapError> {
        let allocation = self
            .allocations
            .get_mut(&pixel)
            .ok_or(ColormapError::ColorNotAllocated(pixel))?;

        if allocation.client != client {
            return Err(ColormapError::AccessDenied);
        }

        if allocation.ref_count > 1 {
            allocation.ref_count -= 1;
        } else {
            self.allocations.remove(&pixel);
            if !self.is_default || pixel >= 2 {
                // Don't remove default colors
                self.entries.remove(&pixel);
            }
        }

        Ok(())
    }

    /// Query a color by pixel value
    pub fn query_color(&self, pixel: u32) -> Result<Color, ColormapError> {
        self.entries
            .get(&pixel)
            .copied()
            .ok_or(ColormapError::InvalidPixel(pixel))
    }

    /// Free all colors allocated by a client
    pub fn free_client_colors(&mut self, client: ClientId) -> Result<u32, ColormapError> {
        let pixels_to_free: Vec<u32> = self
            .allocations
            .iter()
            .filter(|(_, alloc)| alloc.client == client)
            .map(|(pixel, _)| *pixel)
            .collect();

        let mut freed_count = 0;
        for pixel in pixels_to_free {
            if self.free_color(pixel, client).is_ok() {
                freed_count += 1;
            }
        }

        Ok(freed_count)
    }

    /// Get all allocated pixels for a client
    pub fn get_client_allocations(&self, client: ClientId) -> Vec<u32> {
        self.allocations
            .iter()
            .filter(|(_, alloc)| alloc.client == client)
            .map(|(pixel, _)| *pixel)
            .collect()
    }

    /// Get allocation info for a pixel
    pub fn get_allocation_info(&self, pixel: u32) -> Option<&ColorAllocation> {
        self.allocations.get(&pixel)
    }

    /// Initialize colormap entries based on visual class
    fn initialize_entries(&mut self) -> Result<(), LifecycleError> {
        match self.visual_class {
            VisualClass::StaticGray => self.init_static_gray(),
            VisualClass::GrayScale => self.init_grayscale(),
            VisualClass::StaticColor => self.init_static_color(),
            VisualClass::PseudoColor => self.init_pseudocolor(),
            VisualClass::TrueColor => self.init_truecolor(),
            VisualClass::DirectColor => self.init_directcolor(),
        }
    }

    /// Initialize static grayscale colormap
    fn init_static_gray(&mut self) -> Result<(), LifecycleError> {
        for i in 0..self.size {
            let intensity = (i * 65535 / (self.size - 1)) as u16;
            let color = Color::new(intensity, intensity, intensity);
            self.entries.insert(i, color);
        }
        Ok(())
    }

    /// Initialize dynamic grayscale colormap
    fn init_grayscale(&mut self) -> Result<(), LifecycleError> {
        // Start with a basic grayscale ramp
        self.init_static_gray()
    }

    /// Initialize static color colormap
    fn init_static_color(&mut self) -> Result<(), LifecycleError> {
        // Create a default color cube
        self.create_color_cube()
    }

    /// Initialize pseudocolor colormap
    fn init_pseudocolor(&mut self) -> Result<(), LifecycleError> {
        // Start with a color cube, but allow dynamic changes
        self.create_color_cube()
    }

    /// Initialize TrueColor colormap
    fn init_truecolor(&mut self) -> Result<(), LifecycleError> {
        // TrueColor doesn't use a colormap in the traditional sense
        // Colors are computed directly from pixel values
        Ok(())
    }

    /// Initialize DirectColor colormap
    fn init_directcolor(&mut self) -> Result<(), LifecycleError> {
        // DirectColor allows separate control of R, G, B components
        self.create_linear_ramps()
    }

    /// Create a default color cube
    fn create_color_cube(&mut self) -> Result<(), LifecycleError> {
        let cube_size = ((self.size as f64).powf(1.0 / 3.0) as u32).max(1);
        let mut pixel = 0;

        for r in 0..cube_size {
            for g in 0..cube_size {
                for b in 0..cube_size {
                    if pixel >= self.size {
                        break;
                    }

                    let red = (r * 65535 / (cube_size - 1).max(1)) as u16;
                    let green = (g * 65535 / (cube_size - 1).max(1)) as u16;
                    let blue = (b * 65535 / (cube_size - 1).max(1)) as u16;

                    self.entries.insert(pixel, Color::new(red, green, blue));
                    pixel += 1;
                }
            }
        }

        Ok(())
    }

    /// Create linear color ramps for DirectColor
    fn create_linear_ramps(&mut self) -> Result<(), LifecycleError> {
        for i in 0..self.size {
            let intensity = (i * 65535 / (self.size - 1).max(1)) as u16;
            self.entries
                .insert(i, Color::new(intensity, intensity, intensity));
        }
        Ok(())
    }

    /// Setup default colors (black and white at pixels 0 and 1)
    fn setup_default_colors(&mut self) -> Result<(), LifecycleError> {
        self.entries.insert(0, Color::BLACK);
        self.entries.insert(1, Color::WHITE);

        // Mark as allocated but with no specific client
        self.allocations.insert(
            0,
            ColorAllocation {
                pixel: 0,
                color: Color::BLACK,
                client: 0, // System allocation
                read_only: true,
                ref_count: u32::MAX, // Never freed
            },
        );

        self.allocations.insert(
            1,
            ColorAllocation {
                pixel: 1,
                color: Color::WHITE,
                client: 0, // System allocation
                read_only: true,
                ref_count: u32::MAX, // Never freed
            },
        );

        self.next_pixel = 2;
        Ok(())
    }

    /// Find the next available pixel value
    fn find_available_pixel(&mut self) -> Result<u32, ColormapError> {
        let start_pixel = self.next_pixel;

        for _ in 0..self.size {
            if !self.allocations.contains_key(&self.next_pixel) {
                let pixel = self.next_pixel;
                self.next_pixel = (self.next_pixel + 1) % self.size;
                return Ok(pixel);
            }
            self.next_pixel = (self.next_pixel + 1) % self.size;
        }

        // Reset and try again from the beginning (skipping reserved colors)
        self.next_pixel = if self.is_default { 2 } else { 0 };

        if self.next_pixel != start_pixel {
            return self.find_available_pixel();
        }

        Err(ColormapError::NoMoreColors)
    }
}

impl Resource for ColormapResource {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Colormap
    }

    fn xid(&self) -> XId {
        self.xid
    }

    fn owner(&self) -> ClientId {
        self.owner
    }

    fn prepare_destroy(&mut self) -> Result<(), LifecycleError> {
        // Don't allow destruction of the default colormap
        if self.is_default {
            return Err(LifecycleError::FinalizationFailed(
                "Cannot destroy default colormap".into(),
            ));
        }

        // Clear all allocations and entries
        self.allocations.clear();
        self.entries.clear();

        Ok(())
    }

    fn dependencies(&self) -> Vec<XId> {
        vec![self.visual]
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
    fn test_colormap_creation() {
        let colormap = ColormapResource::new(100, 1, 50, VisualClass::PseudoColor, 256).unwrap();
        assert_eq!(colormap.xid(), 100);
        assert_eq!(colormap.owner(), 1);
        assert_eq!(colormap.visual(), 50);
        assert_eq!(colormap.visual_class(), VisualClass::PseudoColor);
        assert_eq!(colormap.size(), 256);
        assert_eq!(colormap.resource_type(), ResourceType::Colormap);
    }

    #[test]
    fn test_default_colormap() {
        let colormap =
            ColormapResource::new_default(100, 50, VisualClass::PseudoColor, 256).unwrap();
        assert!(colormap.is_default());

        // Check default colors
        assert_eq!(colormap.query_color(0).unwrap(), Color::BLACK);
        assert_eq!(colormap.query_color(1).unwrap(), Color::WHITE);
    }

    #[test]
    fn test_color_allocation() {
        let mut colormap =
            ColormapResource::new(100, 1, 50, VisualClass::PseudoColor, 256).unwrap();

        let red_color = Color::RED;
        let pixel = colormap.alloc_color(red_color, 1).unwrap();

        assert_eq!(colormap.query_color(pixel).unwrap(), red_color);

        let allocation = colormap.get_allocation_info(pixel).unwrap();
        assert_eq!(allocation.client, 1);
        assert!(allocation.read_only);
        assert_eq!(allocation.ref_count, 1);
    }

    #[test]
    fn test_color_free() {
        let mut colormap =
            ColormapResource::new(100, 1, 50, VisualClass::PseudoColor, 256).unwrap();

        let pixel = colormap.alloc_color(Color::BLUE, 1).unwrap();
        assert!(colormap.get_allocation_info(pixel).is_some());

        colormap.free_color(pixel, 1).unwrap();
        assert!(colormap.get_allocation_info(pixel).is_none());
    }

    #[test]
    fn test_read_write_allocation() {
        let mut colormap =
            ColormapResource::new(100, 1, 50, VisualClass::PseudoColor, 256).unwrap();

        let pixel = colormap.alloc_color_rw(1).unwrap();

        // Should be able to store a new color
        colormap.store_color(pixel, Color::GREEN, 1).unwrap();
        assert_eq!(colormap.query_color(pixel).unwrap(), Color::GREEN);
    }

    #[test]
    fn test_client_color_cleanup() {
        let mut colormap =
            ColormapResource::new(100, 1, 50, VisualClass::PseudoColor, 256).unwrap();

        colormap.alloc_color(Color::RED, 1).unwrap();
        colormap.alloc_color(Color::GREEN, 1).unwrap();
        colormap.alloc_color(Color::BLUE, 2).unwrap();

        let freed = colormap.free_client_colors(1).unwrap();
        assert_eq!(freed, 2);

        let client1_allocs = colormap.get_client_allocations(1);
        assert_eq!(client1_allocs.len(), 0);

        let client2_allocs = colormap.get_client_allocations(2);
        assert_eq!(client2_allocs.len(), 1);
    }
    #[test]
    fn test_color_creation() {
        let color = Color::new(255 * 257, 128 * 257, 0);
        assert_eq!(color.red, 255 * 257);
        assert_eq!(color.green, 128 * 257);
        assert_eq!(color.blue, 0);
    }
}
