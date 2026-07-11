//! RANDR (Resize and Rotate) Extension Implementation
//!
//! This module implements the X11 RANDR extension which provides:
//! - Display configuration management
//! - Resolution and refresh rate changes
//! - Multi-monitor support
//! - Display rotation and reflection
//! - Hotplug event notifications

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod handlers;

pub use handlers::*;

/// RANDR Mode represents a display mode (resolution + refresh rate)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Mode {
    pub id: u32,
    pub width: u16,
    pub height: u16,
    pub refresh_rate: u32, // in mHz (millihertz)
    pub name: String,
}

/// RANDR Output represents a physical or virtual display output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub id: u32,
    pub name: String,
    pub crtc_id: Option<u32>,        // Currently assigned CRTC, if any
    pub modes: Vec<u32>,             // List of supported mode IDs
    pub preferred_mode: Option<u32>, // Preferred mode ID
    pub connected: bool,             // Whether output is connected
    pub width_mm: u32,               // Physical width in millimeters
    pub height_mm: u32,              // Physical height in millimeters
}

/// RANDR CRTC (Controller) manages the display of content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crtc {
    pub id: u32,
    pub x: i16, // Position on screen
    pub y: i16,
    pub width: u16, // Current mode dimensions
    pub height: u16,
    pub mode_id: Option<u32>, // Currently active mode
    pub rotation: Rotation,   // Current rotation/reflection
    pub outputs: Vec<u32>,    // Connected output IDs
}

/// Display rotation and reflection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    Rotate0 = 1,
    Rotate90 = 2,
    Rotate180 = 4,
    Rotate270 = 8,
    ReflectX = 16,
    ReflectY = 32,
}

impl Rotation {
    pub fn from_u16(value: u16) -> Self {
        match value {
            1 => Rotation::Rotate0,
            2 => Rotation::Rotate90,
            4 => Rotation::Rotate180,
            8 => Rotation::Rotate270,
            16 => Rotation::ReflectX,
            32 => Rotation::ReflectY,
            _ => Rotation::Rotate0, // Default
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            Rotation::Rotate0 => 1,
            Rotation::Rotate90 => 2,
            Rotation::Rotate180 => 4,
            Rotation::Rotate270 => 8,
            Rotation::ReflectX => 16,
            Rotation::ReflectY => 32,
        }
    }
}

/// RANDR Screen represents the root window's display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screen {
    pub min_width: u16,
    pub min_height: u16,
    pub max_width: u16,
    pub max_height: u16,
    pub width: u16,  // Current width
    pub height: u16, // Current height
    pub modes: Vec<Mode>,
    pub outputs: Vec<Output>,
    pub crtcs: Vec<Crtc>,
}

/// RANDR state for the entire server
#[derive(Debug, Clone)]
pub struct RandrState {
    pub version_major: u32,
    pub version_minor: u32,
    pub screens: HashMap<u32, Screen>, // Screen ID -> Screen
}

impl Default for RandrState {
    fn default() -> Self {
        Self {
            version_major: 1,
            version_minor: 6, // Latest stable version
            screens: HashMap::new(),
        }
    }
}

impl RandrState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize RANDR state for a screen
    pub fn init_screen(&mut self, screen_id: u32, width: u16, height: u16) {
        let mut screen = Screen {
            min_width: 320,
            min_height: 240,
            max_width: 8192,
            max_height: 8192,
            width,
            height,
            modes: Vec::new(),
            outputs: Vec::new(),
            crtcs: Vec::new(),
        };

        // Add some common modes
        let common_modes = vec![
            Mode {
                id: 1,
                width: 640,
                height: 480,
                refresh_rate: 60000,
                name: "640x480".to_string(),
            },
            Mode {
                id: 2,
                width: 800,
                height: 600,
                refresh_rate: 60000,
                name: "800x600".to_string(),
            },
            Mode {
                id: 3,
                width: 1024,
                height: 768,
                refresh_rate: 60000,
                name: "1024x768".to_string(),
            },
            Mode {
                id: 4,
                width,
                height,
                refresh_rate: 60000,
                name: format!("{}x{}", width, height),
            },
            Mode {
                id: 5,
                width: 1920,
                height: 1080,
                refresh_rate: 60000,
                name: "1920x1080".to_string(),
            },
        ];

        screen.modes = common_modes;

        // Add a primary output
        let output = Output {
            id: 1,
            name: "Virtual-1".to_string(),
            crtc_id: Some(1),
            modes: screen.modes.iter().map(|m| m.id).collect(),
            preferred_mode: Some(4), // Current resolution
            connected: true,
            width_mm: 400,
            height_mm: 225,
        };
        screen.outputs.push(output);

        // Add a CRTC
        let crtc = Crtc {
            id: 1,
            x: 0,
            y: 0,
            width,
            height,
            mode_id: Some(4),
            rotation: Rotation::Rotate0,
            outputs: vec![1],
        };
        screen.crtcs.push(crtc);

        self.screens.insert(screen_id, screen);
    }

    /// Get screen by ID
    pub fn get_screen(&self, screen_id: u32) -> Option<&Screen> {
        self.screens.get(&screen_id)
    }

    /// Get mutable screen by ID
    pub fn get_screen_mut(&mut self, screen_id: u32) -> Option<&mut Screen> {
        self.screens.get_mut(&screen_id)
    }
}
