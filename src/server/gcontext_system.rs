// gcontext_system.rs
//! Graphics Context (GC) management for X11 server

use crate::protocol::GContextId;
use crate::server::client_system::ClientId;
use std::collections::HashMap;
use tracing::debug;

/// Graphics context attributes
#[derive(Debug, Clone)]
pub struct GraphicsContext {
    pub id: GContextId,
    pub drawable: Option<u32>, // The drawable this GC is associated with
    pub function: u8,          // Raster operation function
    pub plane_mask: u32,       // Plane mask
    pub foreground: u32,       // Foreground color
    pub background: u32,       // Background color
    pub line_width: u16,       // Line width
    pub line_style: u8,        // Line style
    pub cap_style: u8,         // Cap style
    pub join_style: u8,        // Join style
    pub fill_style: u8,        // Fill style
    pub fill_rule: u8,         // Fill rule
    pub arc_mode: u8,          // Arc mode
    pub tile: Option<u32>,     // Tile pixmap
    pub stipple: Option<u32>,  // Stipple pixmap
    pub tile_stipple_x_origin: i16, // Tile/stipple X origin
    pub tile_stipple_y_origin: i16, // Tile/stipple Y origin
    pub font: Option<u32>,     // Font
    pub subwindow_mode: u8,    // Subwindow mode
    pub graphics_exposures: bool, // Graphics exposures
    pub clip_x_origin: i16,    // Clip X origin
    pub clip_y_origin: i16,    // Clip Y origin
    pub clip_mask: Option<u32>, // Clip mask
    pub dash_offset: u16,      // Dash offset
    pub dashes: u8,            // Dashes
    pub owner: ClientId,       // Client that owns this GC
}

impl GraphicsContext {
    pub fn new(id: GContextId, drawable: u32, owner: ClientId) -> Self {
        Self {
            id,
            drawable: Some(drawable),
            function: 3, // Copy
            plane_mask: 0xFFFF_FFFF,
            foreground: 0,        // Black
            background: 0xFFFFFF, // White
            line_width: 0,
            line_style: 0, // Solid
            cap_style: 1,  // Butt
            join_style: 0, // Miter
            fill_style: 0, // Solid
            fill_rule: 0,  // EvenOdd
            arc_mode: 0,   // Chord
            tile: None,
            stipple: None,
            tile_stipple_x_origin: 0,
            tile_stipple_y_origin: 0,
            font: None,
            subwindow_mode: 0, // ClipByChildren
            graphics_exposures: true,
            clip_x_origin: 0,
            clip_y_origin: 0,
            clip_mask: None,
            dash_offset: 0,
            dashes: 4,
            owner,
        }
    }
}

/// Manages X11 graphics contexts
#[derive(Debug)]
pub struct GraphicsContextSystem {
    gcontexts: HashMap<GContextId, GraphicsContext>,
}

impl GraphicsContextSystem {
    pub fn new() -> Self {
        Self {
            gcontexts: HashMap::new(),
        }
    }

    pub fn create_gc(
        &mut self,
        id: GContextId,
        drawable: u32,
        owner: ClientId,
    ) -> Result<(), String> {
        if self.gcontexts.contains_key(&id) {
            return Err(format!("Graphics context ID {} already exists", id));
        }

        let gc = GraphicsContext::new(id, drawable, owner);
        self.gcontexts.insert(id, gc);

        debug!(
            "Created graphics context {} for drawable {} owned by client {}",
            id, drawable, owner
        );
        Ok(())
    }

    pub fn destroy_gc(&mut self, id: GContextId) -> Result<(), String> {
        if !self.gcontexts.contains_key(&id) {
            return Err(format!("Graphics context {} does not exist", id));
        }

        self.gcontexts.remove(&id);
        debug!("Destroyed graphics context {}", id);
        Ok(())
    }

    pub fn get_gc(&self, id: GContextId) -> Option<&GraphicsContext> {
        self.gcontexts.get(&id)
    }

    pub fn get_gc_mut(&mut self, id: GContextId) -> Option<&mut GraphicsContext> {
        self.gcontexts.get_mut(&id)
    }

    pub fn gc_exists(&self, id: GContextId) -> bool {
        self.gcontexts.contains_key(&id)
    }

    /// Remove all GCs owned by a client
    pub fn cleanup_client_gcs(&mut self, client_id: ClientId) {
        let count = self.gcontexts.len();
        self.gcontexts.retain(|_, gc| gc.owner != client_id);
        let removed = count - self.gcontexts.len();
        if removed > 0 {
            debug!(
                "Removed {} graphics contexts for client {}",
                removed, client_id
            );
        }
    }
}

impl Default for GraphicsContextSystem {
    fn default() -> Self {
        Self::new()
    }
}
