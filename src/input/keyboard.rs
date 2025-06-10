//! Keyboard input handling
//!
//! This module handles keyboard events, key mapping, and keyboard state.

use crate::protocol::types::*;
use crate::protocol::responses::{Event, KeyPressEvent, KeyReleaseEvent};
use crate::Result;

/// Keyboard state manager
pub struct KeyboardManager {
    /// Current keyboard state (pressed keys)
    key_state: [bool; 256],
    /// Key repeat settings
    repeat_delay: u32,
    /// Key repeat rate
    repeat_rate: u32,
    /// Auto-repeat enabled keys
    auto_repeat: [bool; 256],
}

impl KeyboardManager {
    /// Create a new keyboard manager
    pub fn new() -> Self {
        Self {
            key_state: [false; 256],
            repeat_delay: 500, // 500ms
            repeat_rate: 30,   // 30 Hz
            auto_repeat: [true; 256], // All keys auto-repeat by default
        }
    }    /// Process a key press event
    pub fn key_press(&mut self, keycode: KeyCode, timestamp: Timestamp) -> Result<Event> {
        self.key_state[keycode as usize] = true;

        let event = KeyPressEvent {
            detail: keycode,
            time: timestamp,
            root: 1, // Root window
            event: 1, // TODO: Determine target window
            child: 0,
            root_x: 0, // TODO: Get cursor position
            root_y: 0,
            event_x: 0,
            event_y: 0,
            state: ModifierMask::from_bits_truncate(self.get_modifier_state()),
            same_screen: true,
        };

        Ok(Event::KeyPress(event))
    }

    /// Process a key release event
    pub fn key_release(&mut self, keycode: KeyCode, timestamp: Timestamp) -> Result<Event> {
        self.key_state[keycode as usize] = false;

        let event = KeyReleaseEvent {
            detail: keycode,
            time: timestamp,
            root: 1,
            event: 1,
            child: 0,
            root_x: 0,
            root_y: 0,
            event_x: 0,
            event_y: 0,
            state: ModifierMask::from_bits_truncate(self.get_modifier_state()),
            same_screen: true,
        };

        Ok(Event::KeyRelease(event))
    }    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        self.key_state[keycode as usize]
    }

    /// Get the current modifier state
    pub fn get_modifier_state(&self) -> u16 {
        let mut state = 0u16;

        // Check common modifier keys
        if self.is_key_pressed(KEYCODE_SHIFT_L) || self.is_key_pressed(KEYCODE_SHIFT_R) {
            state |= MODIFIER_SHIFT;
        }
        if self.is_key_pressed(KEYCODE_CTRL_L) || self.is_key_pressed(KEYCODE_CTRL_R) {
            state |= MODIFIER_CONTROL;
        }
        if self.is_key_pressed(KEYCODE_ALT_L) || self.is_key_pressed(KEYCODE_ALT_R) {
            state |= MODIFIER_MOD1;
        }
        if self.is_key_pressed(KEYCODE_CAPS_LOCK) {
            state |= MODIFIER_LOCK;
        }

        state
    }

    /// Set key repeat settings
    pub fn set_repeat_settings(&mut self, delay: u32, rate: u32) {
        self.repeat_delay = delay;
        self.repeat_rate = rate;
    }

    /// Enable or disable auto-repeat for a key
    pub fn set_auto_repeat(&mut self, keycode: KeyCode, enabled: bool) {
        self.auto_repeat[keycode as usize] = enabled;
    }

    /// Check if auto-repeat is enabled for a key
    pub fn is_auto_repeat_enabled(&self, keycode: KeyCode) -> bool {
        self.auto_repeat[keycode as usize]
    }

    /// Convert keycode to keysym (basic mapping)
    pub fn keycode_to_keysym(&self, keycode: KeyCode) -> KeySym {
        // TODO: Implement proper keycode to keysym mapping
        // This is a very basic mapping for demonstration
        match keycode {
            KEYCODE_A => KEYSYM_A,
            KEYCODE_B => KEYSYM_B,
            KEYCODE_C => KEYSYM_C,
            KEYCODE_SPACE => KEYSYM_SPACE,
            KEYCODE_ENTER => KEYSYM_RETURN,
            KEYCODE_ESC => KEYSYM_ESCAPE,
            _ => keycode as KeySym,
        }
    }

    /// Convert keysym to keycode (reverse mapping)
    pub fn keysym_to_keycode(&self, keysym: KeySym) -> Option<KeyCode> {
        // TODO: Implement proper keysym to keycode mapping
        match keysym {
            KEYSYM_A => Some(KEYCODE_A),
            KEYSYM_B => Some(KEYCODE_B),
            KEYSYM_C => Some(KEYCODE_C),
            KEYSYM_SPACE => Some(KEYCODE_SPACE),
            KEYSYM_RETURN => Some(KEYCODE_ENTER),
            KEYSYM_ESCAPE => Some(KEYCODE_ESC),
            _ => None,
        }
    }
}

// Common keycode constants
pub const KEYCODE_ESC: KeyCode = 9;
pub const KEYCODE_1: KeyCode = 10;
pub const KEYCODE_2: KeyCode = 11;
pub const KEYCODE_3: KeyCode = 12;
pub const KEYCODE_4: KeyCode = 13;
pub const KEYCODE_5: KeyCode = 14;
pub const KEYCODE_6: KeyCode = 15;
pub const KEYCODE_7: KeyCode = 16;
pub const KEYCODE_8: KeyCode = 17;
pub const KEYCODE_9: KeyCode = 18;
pub const KEYCODE_0: KeyCode = 19;
pub const KEYCODE_Q: KeyCode = 24;
pub const KEYCODE_W: KeyCode = 25;
pub const KEYCODE_E: KeyCode = 26;
pub const KEYCODE_R: KeyCode = 27;
pub const KEYCODE_T: KeyCode = 28;
pub const KEYCODE_Y: KeyCode = 29;
pub const KEYCODE_U: KeyCode = 30;
pub const KEYCODE_I: KeyCode = 31;
pub const KEYCODE_O: KeyCode = 32;
pub const KEYCODE_P: KeyCode = 33;
pub const KEYCODE_A: KeyCode = 38;
pub const KEYCODE_S: KeyCode = 39;
pub const KEYCODE_D: KeyCode = 40;
pub const KEYCODE_F: KeyCode = 41;
pub const KEYCODE_G: KeyCode = 42;
pub const KEYCODE_H: KeyCode = 43;
pub const KEYCODE_J: KeyCode = 44;
pub const KEYCODE_K: KeyCode = 45;
pub const KEYCODE_L: KeyCode = 46;
pub const KEYCODE_Z: KeyCode = 52;
pub const KEYCODE_X: KeyCode = 53;
pub const KEYCODE_C: KeyCode = 54;
pub const KEYCODE_V: KeyCode = 55;
pub const KEYCODE_B: KeyCode = 56;
pub const KEYCODE_N: KeyCode = 57;
pub const KEYCODE_M: KeyCode = 58;
pub const KEYCODE_SPACE: KeyCode = 65;
pub const KEYCODE_CAPS_LOCK: KeyCode = 66;
pub const KEYCODE_SHIFT_L: KeyCode = 50;
pub const KEYCODE_SHIFT_R: KeyCode = 62;
pub const KEYCODE_CTRL_L: KeyCode = 37;
pub const KEYCODE_CTRL_R: KeyCode = 105;
pub const KEYCODE_ALT_L: KeyCode = 64;
pub const KEYCODE_ALT_R: KeyCode = 108;
pub const KEYCODE_ENTER: KeyCode = 36;

// Common keysym constants
pub const KEYSYM_A: KeySym = 0x0061;
pub const KEYSYM_B: KeySym = 0x0062;
pub const KEYSYM_C: KeySym = 0x0063;
pub const KEYSYM_SPACE: KeySym = 0x0020;
pub const KEYSYM_RETURN: KeySym = 0xFF0D;
pub const KEYSYM_ESCAPE: KeySym = 0xFF1B;

// Modifier state constants
pub const MODIFIER_SHIFT: u16 = 1;
pub const MODIFIER_LOCK: u16 = 2;
pub const MODIFIER_CONTROL: u16 = 4;
pub const MODIFIER_MOD1: u16 = 8;
pub const MODIFIER_MOD2: u16 = 16;
pub const MODIFIER_MOD3: u16 = 32;
pub const MODIFIER_MOD4: u16 = 64;
pub const MODIFIER_MOD5: u16 = 128;
