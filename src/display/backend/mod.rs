//! Display backend implementations

// TODO: Add hardware backend when needed
// pub mod hardware;
pub mod headless;
// TODO: Add software backend when needed
// pub mod software;

pub mod manager;
pub mod traits;

// Re-export traits and core types
pub use manager::BackendManager;
pub use traits::DisplayBackend;

/// Backend type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BackendType {
    Headless,
    Software,
    Hardware,
    Virtual,
    Native,
}

impl BackendType {
    pub fn name(&self) -> &'static str {
        match self {
            BackendType::Headless => "headless",
            BackendType::Software => "software",
            BackendType::Hardware => "hardware",
            BackendType::Virtual => "virtual",
            BackendType::Native => "native",
        }
    }
}

/// Backend capabilities
#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub supports_acceleration: bool,
    pub supports_multiple_displays: bool,
    pub max_resolution: crate::display::types::Resolution,
    pub supported_color_depths: Vec<crate::display::types::ColorDepth>,
}

impl Default for BackendCapabilities {
    fn default() -> Self {
        Self {
            supports_acceleration: false,
            supports_multiple_displays: false,
            max_resolution: crate::display::types::Resolution::new(1920, 1080),
            supported_color_depths: vec![
                crate::display::types::ColorDepth::Depth24,
                crate::display::types::ColorDepth::Depth32,
            ],
        }
    }
}
