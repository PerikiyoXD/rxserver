use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphicsBackend {
    Software,
    OpenGL,
    Vulkan,
}

// Fromstr for graphics backend
impl std::str::FromStr for GraphicsBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "software" => Ok(GraphicsBackend::Software),
            "opengl" => Ok(GraphicsBackend::OpenGL),
            "vulkan" => Ok(GraphicsBackend::Vulkan),
            _ => Err(format!("Unknown graphics backend: {}", s)),
        }
    }
}

// To Display
impl Display for GraphicsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}