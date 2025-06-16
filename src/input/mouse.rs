//! Mouse input handling
//!
//! This module handles mouse events, button presses, and cursor movement.

use crate::core::error::ServerResult;

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left = 1,
    Middle = 2,
    Right = 3,
    WheelUp = 4,
    WheelDown = 5,
}

/// Button mask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonMask(u16);

impl ButtonMask {
    pub const BUTTON1: ButtonMask = ButtonMask(1 << 8); // Left
    pub const BUTTON2: ButtonMask = ButtonMask(1 << 9); // Middle
    pub const BUTTON3: ButtonMask = ButtonMask(1 << 10); // Right
    pub const BUTTON4: ButtonMask = ButtonMask(1 << 11); // Wheel up
    pub const BUTTON5: ButtonMask = ButtonMask(1 << 12); // Wheel down

    pub fn contains(&self, other: ButtonMask) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn insert(&mut self, other: ButtonMask) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: ButtonMask) {
        self.0 &= !other.0;
    }
}

/// Mouse event
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub x: i16,
    pub y: i16,
    pub button: Option<MouseButton>,
    pub pressed: bool,
    pub button_mask: ButtonMask,
    pub time: u32,
}

/// Mouse manager
pub struct MouseManager {
    x: i16,
    y: i16,
    button_state: ButtonMask,
    acceleration: f32,
    sensitivity: f32,
}

impl MouseManager {
    /// Create a new mouse manager
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            button_state: ButtonMask(0),
            acceleration: 1.0,
            sensitivity: 1.0,
        }
    }

    /// Process mouse movement
    pub fn mouse_move(&mut self, x: i16, y: i16) -> ServerResult<MouseEvent> {
        self.x = x;
        self.y = y;

        Ok(MouseEvent {
            x: self.x,
            y: self.y,
            button: None,
            pressed: false,
            button_mask: self.button_state,
            time: 0, // TODO: Get actual timestamp
        })
    }

    /// Process mouse button press
    pub fn button_press(&mut self, button: MouseButton) -> ServerResult<MouseEvent> {
        let button_mask = match button {
            MouseButton::Left => ButtonMask::BUTTON1,
            MouseButton::Middle => ButtonMask::BUTTON2,
            MouseButton::Right => ButtonMask::BUTTON3,
            MouseButton::WheelUp => ButtonMask::BUTTON4,
            MouseButton::WheelDown => ButtonMask::BUTTON5,
        };

        self.button_state.insert(button_mask);

        Ok(MouseEvent {
            x: self.x,
            y: self.y,
            button: Some(button),
            pressed: true,
            button_mask: self.button_state,
            time: 0, // TODO: Get actual timestamp
        })
    }

    /// Process mouse button release
    pub fn button_release(&mut self, button: MouseButton) -> ServerResult<MouseEvent> {
        let button_mask = match button {
            MouseButton::Left => ButtonMask::BUTTON1,
            MouseButton::Middle => ButtonMask::BUTTON2,
            MouseButton::Right => ButtonMask::BUTTON3,
            MouseButton::WheelUp => ButtonMask::BUTTON4,
            MouseButton::WheelDown => ButtonMask::BUTTON5,
        };

        self.button_state.remove(button_mask);

        Ok(MouseEvent {
            x: self.x,
            y: self.y,
            button: Some(button),
            pressed: false,
            button_mask: self.button_state,
            time: 0, // TODO: Get actual timestamp
        })
    }

    /// Get current mouse position
    pub fn get_position(&self) -> (i16, i16) {
        (self.x, self.y)
    }

    /// Check if a button is currently pressed
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        let button_mask = match button {
            MouseButton::Left => ButtonMask::BUTTON1,
            MouseButton::Middle => ButtonMask::BUTTON2,
            MouseButton::Right => ButtonMask::BUTTON3,
            MouseButton::WheelUp => ButtonMask::BUTTON4,
            MouseButton::WheelDown => ButtonMask::BUTTON5,
        };

        self.button_state.contains(button_mask)
    }

    /// Get current button state
    pub fn get_button_state(&self) -> ButtonMask {
        self.button_state
    }

    /// Set mouse acceleration and sensitivity
    pub fn set_params(&mut self, acceleration: f32, sensitivity: f32) {
        self.acceleration = acceleration;
        self.sensitivity = sensitivity;
    }

    /// Warp mouse cursor to position
    pub fn warp_cursor(&mut self, x: i16, y: i16) -> ServerResult<()> {
        self.x = x;
        self.y = y;
        Ok(())
    }
}

impl Default for MouseManager {
    fn default() -> Self {
        Self::new()
    }
}
