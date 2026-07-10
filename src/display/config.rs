use serde::{Deserialize, Serialize};

use super::types::DisplayKind;

/// Display configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisplayConfig {
    pub id: usize,
    pub kind: DisplayKind,
    pub name: String,
    pub resolution: [u32; 2],
    /// Draw a dev-time overlay (window count/status indicators) on top of
    /// the rendered display. Off by default so the virtual display matches
    /// what a real X server's root window looks like.
    #[serde(default)]
    pub debug_overlay: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            id: 0,
            kind: DisplayKind::Virtual,
            name: "Default Virtual Display".to_string(),
            resolution: [640, 480],
            debug_overlay: false,
        }
    }
}
