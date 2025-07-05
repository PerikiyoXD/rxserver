use serde::{Deserialize, Serialize};

use crate::transport::TransportKind;

use super::types::DisplayType;

/// Display configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisplayConfig {
    pub id: usize,
    pub r#type: DisplayType,
    pub transport: TransportKind,
    pub name: String,
    pub resolution: [u32; 2],
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            id: 0,
            r#type: DisplayType::Virtual,
            name: "Default Virtual Display".to_string(),
            transport: TransportKind::Tcp,
            resolution: [640, 480],
        }
    }
}
