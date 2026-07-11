// picture_system.rs
use crate::protocol::PictureId;
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

/// What a Picture actually is. Only solid fills exist so far - Pictures
/// backed by a real drawable (the common case, created via
/// `RenderCreatePicture`) are future work.
#[derive(Debug, Clone)]
pub enum PictureContent {
    SolidFill(RenderColor),
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

    pub fn get_picture(&self, id: PictureId) -> Option<&Picture> {
        self.pictures.get(&id)
    }

    pub fn remove_picture(&mut self, id: PictureId) -> bool {
        self.pictures.remove(&id).is_some()
    }
}
