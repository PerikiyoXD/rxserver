//! Input configuration types

use serde::{Deserialize, Serialize};

/// Input system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfiguration {
    /// Enable mouse input
    pub enable_mouse: bool,
    /// Enable keyboard input  
    pub enable_keyboard: bool,
    /// Enable touch input
    pub enable_touch: bool,
    /// Mouse sensitivity
    pub mouse_sensitivity: f32,
    /// Keyboard repeat rate
    pub keyboard_repeat_rate: u32,
}

impl Default for InputConfiguration {
    fn default() -> Self {
        Self {
            enable_mouse: true,
            enable_keyboard: true,
            enable_touch: true,
            mouse_sensitivity: 1.0,
            keyboard_repeat_rate: 30,
        }
    }
}

/// Input device representation
#[derive(Debug, Clone)]
pub struct InputDevice {
    pub id: u32,
    pub name: String,
    pub device_type: DeviceType,
    pub capabilities: DeviceCapabilities,
}

/// Types of input devices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Mouse,
    Keyboard,
    Touchpad,
    Touchscreen,
    Tablet,
    Joystick,
    Other,
}

/// Device capabilities
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    pub has_buttons: bool,
    pub has_axes: bool,
    pub has_keys: bool,
    pub button_count: u8,
    pub axis_count: u8,
}

/// Input event representation
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPress {
        device_id: u32,
        keycode: u32,
        timestamp: u64,
    },
    KeyRelease {
        device_id: u32,
        keycode: u32,
        timestamp: u64,
    },
    MouseMove {
        device_id: u32,
        x: i16,
        y: i16,
        timestamp: u64,
    },
    MouseButton {
        device_id: u32,
        button: u8,
        pressed: bool,
        timestamp: u64,
    },
    MouseWheel {
        device_id: u32,
        delta_x: i16,
        delta_y: i16,
        timestamp: u64,
    },
}
