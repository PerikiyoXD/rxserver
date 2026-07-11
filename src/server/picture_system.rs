// picture_system.rs
use crate::protocol::{DrawableId, PictureId, PixmapId};
use crate::server::client_system::ClientId;
use std::collections::HashMap;
use tracing::debug;

/// A RENDER color, 16-bit components straddling the whole 0..=0xFFFF range
/// (matches the wire format of `xRenderColor`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub alpha: u16,
}

/// Picture attributes settable via `RenderCreatePicture`'s value-list and
/// later `RenderChangePicture` - mirrors `GraphicsContext`'s field-per-value
/// shape. Defaults match renderproto.txt's CreatePicture VALUEs table.
#[derive(Debug, Clone)]
pub struct PictureAttributes {
    pub repeat: bool,                  // CPRepeat, default RepeatNone (false)
    pub alpha_map: Option<PictureId>,  // CPAlphaMap, default None
    pub alpha_x_origin: i16,           // CPAlphaXOrigin, default 0
    pub alpha_y_origin: i16,           // CPAlphaYOrigin, default 0
    pub clip_x_origin: i16,            // CPClipXOrigin, default 0
    pub clip_y_origin: i16,            // CPClipYOrigin, default 0
    pub clip_mask: Option<PixmapId>,   // CPClipMask, default None (no clipping)
    pub graphics_exposures: bool,      // CPGraphicsExposure, default true
    pub subwindow_mode: u8,            // CPSubwindowMode, default ClippedByChildren (0)
    pub poly_edge: u8,                 // CPPolyEdge, default Sharp (0)
    pub poly_mode: u8,                 // CPPolyMode, default Precise (0)
    pub dither: u32,                   // CPDither, default 0
    pub component_alpha: bool,         // CPComponentAlpha, default false
}

impl Default for PictureAttributes {
    fn default() -> Self {
        Self {
            repeat: false,
            alpha_map: None,
            alpha_x_origin: 0,
            alpha_y_origin: 0,
            clip_x_origin: 0,
            clip_y_origin: 0,
            clip_mask: None,
            graphics_exposures: true,
            subwindow_mode: 0,
            poly_edge: 0,
            poly_mode: 0,
            dither: 0,
            component_alpha: false,
        }
    }
}

/// What a Picture actually is: either a solid-fill source (no backing
/// drawable) or the common case, a Picture wrapping a real drawable
/// (window or pixmap) created via `RenderCreatePicture`.
#[derive(Debug, Clone)]
pub enum PictureContent {
    SolidFill(RenderColor),
    Drawable {
        drawable: DrawableId,
        format: u32, // PictFormat id, opaque to this server - see RenderQueryPictFormatsHandler
        attributes: PictureAttributes,
    },
}

#[derive(Debug, Clone)]
pub struct Picture {
    pub id: PictureId,
    pub owner: ClientId,
    pub content: PictureContent,
}

/// Manages RENDER extension Picture resources.
#[derive(Debug)]
pub struct PictureSystem {
    pictures: HashMap<PictureId, Picture>,
}

impl PictureSystem {
    pub fn new() -> Self {
        Self {
            pictures: HashMap::new(),
        }
    }

    pub fn create_solid_fill(
        &mut self,
        id: PictureId,
        color: RenderColor,
        owner: ClientId,
    ) -> Result<(), String> {
        if self.pictures.contains_key(&id) {
            return Err(format!("Picture {} already exists", id));
        }

        self.pictures.insert(
            id,
            Picture {
                id,
                owner,
                content: PictureContent::SolidFill(color),
            },
        );
        debug!("Created solid fill picture {} ({:?})", id, color);
        Ok(())
    }

    pub fn create_picture(
        &mut self,
        id: PictureId,
        drawable: DrawableId,
        format: u32,
        attributes: PictureAttributes,
        owner: ClientId,
    ) -> Result<(), String> {
        if self.pictures.contains_key(&id) {
            return Err(format!("Picture {} already exists", id));
        }

        self.pictures.insert(
            id,
            Picture {
                id,
                owner,
                content: PictureContent::Drawable {
                    drawable,
                    format,
                    attributes,
                },
            },
        );
        debug!(
            "Created picture {} for drawable {} (format {})",
            id, drawable, format
        );
        Ok(())
    }

    pub fn get_picture(&self, id: PictureId) -> Option<&Picture> {
        self.pictures.get(&id)
    }

    pub fn remove_picture(&mut self, id: PictureId) -> bool {
        self.pictures.remove(&id).is_some()
    }
}
