// pixmap_system.rs
use crate::protocol::PixmapId;
use crate::server::client_system::ClientId;
use std::collections::HashMap;
use tracing::debug;

/// Internal representation of a pixmap in the X11 server
#[derive(Debug, Clone)]
pub struct Pixmap {
    pub id: PixmapId,
    pub width: u16,
    pub height: u16,
    pub depth: u8,
    pub owner: ClientId,
    pub pixel_data: Vec<u32>, // ARGB pixel data
}

impl Pixmap {
    pub fn new(
        id: PixmapId,
        width: u16,
        height: u16,
        depth: u8,
        owner: ClientId,
    ) -> Result<Self, String> {
        let pixel_count = (width as usize) * (height as usize);
        let pixel_data = vec![0x00000000; pixel_count]; // Initialize with transparent

        Ok(Self {
            id,
            width,
            height,
            depth,
            owner,
            pixel_data,
        })
    }

    /// Set a pixel at the given coordinates. Mirrors `Window::set_pixel` -
    /// drawing operations (PolyFillRectangle, etc.) target either a window
    /// or a pixmap per xproto's DRAWABLE union, so both need the same pixel
    /// write primitive.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u32) {
        if x < self.width && y < self.height {
            let index = (y as usize * self.width as usize) + x as usize;
            if index < self.pixel_data.len() {
                self.pixel_data[index] = color;
            }
        }
    }
}

/// Manages X11 pixmaps
#[derive(Debug)]
pub struct PixmapSystem {
    pixmaps: HashMap<PixmapId, Pixmap>,
}

impl PixmapSystem {
    pub fn new() -> Self {
        Self {
            pixmaps: HashMap::new(),
        }
    }

    /// Create a new pixmap
    pub fn create_pixmap(
        &mut self,
        id: PixmapId,
        width: u16,
        height: u16,
        depth: u8,
        owner: ClientId,
    ) -> Result<(), String> {
        if self.pixmaps.contains_key(&id) {
            return Err(format!("Pixmap {} already exists", id));
        }

        let pixmap = Pixmap::new(id, width, height, depth, owner)?;
        self.pixmaps.insert(id, pixmap);
        debug!("Created pixmap {} ({}x{}x{})", id, width, height, depth);
        Ok(())
    }

    /// Get a pixmap by ID
    pub fn get_pixmap(&self, id: PixmapId) -> Option<&Pixmap> {
        self.pixmaps.get(&id)
    }

    /// Get a mutable pixmap by ID
    pub fn get_pixmap_mut(&mut self, id: PixmapId) -> Option<&mut Pixmap> {
        self.pixmaps.get_mut(&id)
    }

    /// Check if a pixmap exists
    pub fn pixmap_exists(&self, id: PixmapId) -> bool {
        self.pixmaps.contains_key(&id)
    }

    /// Remove a pixmap
    pub fn remove_pixmap(&mut self, id: PixmapId) -> bool {
        self.pixmaps.remove(&id).is_some()
    }
}
