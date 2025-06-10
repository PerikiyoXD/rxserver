//! Mouse input handling
//!
//! This module handles mouse events, cursor position, and button state.

use crate::protocol::events::Event;
use crate::protocol::types::*;
use crate::{todo_low, todo_medium, Result};

/// Mouse state manager
pub struct MouseManager {
    /// Current cursor position
    pub x: i16,
    pub y: i16,
    /// Button state (bit mask)
    button_state: u8,
    /// Mouse acceleration factor
    acceleration: f32,
    /// Last motion timestamp
    last_motion_time: Timestamp,
}

impl MouseManager {
    /// Create a new mouse manager
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            button_state: 0,
            acceleration: 1.0,
            last_motion_time: 0,
        }
    }

    /// Process a mouse button press event
    pub fn button_press(
        &mut self,
        button: u8,
        timestamp: Timestamp,
        window: Window,
    ) -> Result<Event> {
        self.button_state |= 1 << (button - 1);

        Ok(Event::ButtonPress {
            detail: button,
            time: timestamp,
            root: 1, // Root window
            event: window,
            child: 0, // TODO: Determine child window
            root_x: self.x,
            root_y: self.y,
            event_x: self.x, // TODO: Convert to window coordinates
            event_y: self.y,
            state: self.get_button_state(),
            same_screen: true,
        })
    }

    /// Process a mouse button release event
    pub fn button_release(
        &mut self,
        button: u8,
        timestamp: Timestamp,
        window: Window,
    ) -> Result<Event> {
        self.button_state &= !(1 << (button - 1));

        Ok(Event::ButtonRelease {
            detail: button,
            time: timestamp,
            root: 1,
            event: window,
            child: 0,
            root_x: self.x,
            root_y: self.y,
            event_x: self.x,
            event_y: self.y,
            state: self.get_button_state(),
            same_screen: true,
        })
    }

    /// Process a mouse motion event
    pub fn motion(
        &mut self,
        new_x: i16,
        new_y: i16,
        timestamp: Timestamp,
        window: Window,
    ) -> Result<Event> {
        self.x = new_x;
        self.y = new_y;
        self.last_motion_time = timestamp;

        Ok(Event::MotionNotify {
            detail: 0,
            time: timestamp,
            root: 1,
            event: window,
            child: 0,
            root_x: self.x,
            root_y: self.y,
            event_x: self.x,
            event_y: self.y,
            state: self.get_button_state(),
            same_screen: true,
        })
    }

    /// Process a mouse enter event
    pub fn enter_window(&mut self, window: Window, timestamp: Timestamp) -> Result<Event> {
        Ok(Event::EnterNotify {
            detail: NOTIFY_NORMAL,
            time: timestamp,
            root: 1,
            event: window,
            child: 0,
            root_x: self.x,
            root_y: self.y,
            event_x: self.x,
            event_y: self.y,
            state: self.get_button_state(),
            mode: NOTIFY_MODE_NORMAL,
            same_screen_focus: 1,
        })
    }

    /// Process a mouse leave event
    pub fn leave_window(&mut self, window: Window, timestamp: Timestamp) -> Result<Event> {
        Ok(Event::LeaveNotify {
            detail: NOTIFY_NORMAL,
            time: timestamp,
            root: 1,
            event: window,
            child: 0,
            root_x: self.x,
            root_y: self.y,
            event_x: self.x,
            event_y: self.y,
            state: self.get_button_state(),
            mode: NOTIFY_MODE_NORMAL,
            same_screen_focus: 1,
        })
    }

    /// Get the current button state
    pub fn get_button_state(&self) -> u16 {
        self.button_state as u16
    }

    /// Check if a button is currently pressed
    pub fn is_button_pressed(&self, button: u8) -> bool {
        (self.button_state & (1 << (button - 1))) != 0
    }

    /// Set mouse acceleration
    pub fn set_acceleration(&mut self, acceleration: f32) {
        self.acceleration = acceleration;
    }

    /// Get current cursor position
    pub fn get_position(&self) -> (i16, i16) {
        (self.x, self.y)
    }

    /// Set cursor position
    pub fn set_position(&mut self, x: i16, y: i16) {
        self.x = x;
        self.y = y;
    }

    /// Move cursor by relative amount
    pub fn move_relative(&mut self, dx: i16, dy: i16) {
        // Apply acceleration
        let accel_dx = (dx as f32 * self.acceleration) as i16;
        let accel_dy = (dy as f32 * self.acceleration) as i16;

        self.x += accel_dx;
        self.y += accel_dy;
    }

    /// Constrain cursor to bounds
    pub fn constrain_to_bounds(&mut self, width: u16, height: u16) {
        self.x = self.x.max(0).min(width as i16 - 1);
        self.y = self.y.max(0).min(height as i16 - 1);
    }
}

// Mouse button constants
pub const BUTTON_LEFT: u8 = 1;
pub const BUTTON_MIDDLE: u8 = 2;
pub const BUTTON_RIGHT: u8 = 3;
pub const BUTTON_WHEEL_UP: u8 = 4;
pub const BUTTON_WHEEL_DOWN: u8 = 5;

// Notify detail constants
pub const NOTIFY_ANCESTOR: u8 = 0;
pub const NOTIFY_VIRTUAL: u8 = 1;
pub const NOTIFY_INFERIOR: u8 = 2;
pub const NOTIFY_NONLINEAR: u8 = 3;
pub const NOTIFY_NONLINEAR_VIRTUAL: u8 = 4;
pub const NOTIFY_POINTER: u8 = 5;
pub const NOTIFY_POINTER_ROOT: u8 = 6;
pub const NOTIFY_DETAIL_NONE: u8 = 7;
pub const NOTIFY_NORMAL: u8 = 0;

// Notify mode constants
pub const NOTIFY_MODE_NORMAL: u8 = 0;
pub const NOTIFY_MODE_GRAB: u8 = 1;
pub const NOTIFY_MODE_UNGRAB: u8 = 2;
pub const NOTIFY_MODE_WHILE_GRABBED: u8 = 3;
