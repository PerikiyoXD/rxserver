//! Keyboard input handling
//!
//! This module handles keyboard events, key mapping, and keyboard state.

use crate::core::error::ServerResult;

/// Key code type
pub type KeyCode = u8;

/// Key symbol type
pub type KeySym = u32;

/// Modifier mask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModifierMask(u16);

impl ModifierMask {
    pub const SHIFT: ModifierMask = ModifierMask(1 << 0);
    pub const LOCK: ModifierMask = ModifierMask(1 << 1);
    pub const CONTROL: ModifierMask = ModifierMask(1 << 2);
    pub const MOD1: ModifierMask = ModifierMask(1 << 3);
    pub const MOD2: ModifierMask = ModifierMask(1 << 4);
    pub const MOD3: ModifierMask = ModifierMask(1 << 5);
    pub const MOD4: ModifierMask = ModifierMask(1 << 6);
    pub const MOD5: ModifierMask = ModifierMask(1 << 7);

    pub fn contains(&self, other: ModifierMask) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn insert(&mut self, other: ModifierMask) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: ModifierMask) {
        self.0 &= !other.0;
    }
}

/// Keyboard event
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub keycode: KeyCode,
    pub keysym: KeySym,
    pub modifiers: ModifierMask,
    pub pressed: bool,
    pub time: u32,
}

/// Keyboard manager
pub struct KeyboardManager {
    key_state: [bool; 256], // Key press state
    modifier_state: ModifierMask,
    repeat_delay: u32,
    repeat_rate: u32,
}

impl KeyboardManager {
    /// Create a new keyboard manager
    pub fn new() -> Self {
        Self {
            key_state: [false; 256],
            modifier_state: ModifierMask(0),
            repeat_delay: 500, // 500ms default delay
            repeat_rate: 30,   // 30 Hz default rate
        }
    }

    /// Process a key press event
    pub fn key_press(&mut self, keycode: KeyCode) -> ServerResult<KeyEvent> {
        self.key_state[keycode as usize] = true;

        // Update modifier state
        match keycode {
            50 | 62 => self.modifier_state.insert(ModifierMask::SHIFT), // Shift keys
            37 | 105 => self.modifier_state.insert(ModifierMask::CONTROL), // Control keys
            64 | 108 => self.modifier_state.insert(ModifierMask::MOD1), // Alt keys
            _ => {}
        }

        let keysym = self.keycode_to_keysym(keycode);

        Ok(KeyEvent {
            keycode,
            keysym,
            modifiers: self.modifier_state,
            pressed: true,
            time: 0, // TODO: Get actual timestamp
        })
    }

    /// Process a key release event
    pub fn key_release(&mut self, keycode: KeyCode) -> ServerResult<KeyEvent> {
        self.key_state[keycode as usize] = false;

        // Update modifier state
        match keycode {
            50 | 62 => self.modifier_state.remove(ModifierMask::SHIFT), // Shift keys
            37 | 105 => self.modifier_state.remove(ModifierMask::CONTROL), // Control keys
            64 | 108 => self.modifier_state.remove(ModifierMask::MOD1), // Alt keys
            _ => {}
        }

        let keysym = self.keycode_to_keysym(keycode);

        Ok(KeyEvent {
            keycode,
            keysym,
            modifiers: self.modifier_state,
            pressed: false,
            time: 0, // TODO: Get actual timestamp
        })
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        self.key_state[keycode as usize]
    }

    /// Get current modifier state
    pub fn get_modifier_state(&self) -> ModifierMask {
        self.modifier_state
    }

    /// Set keyboard repeat parameters
    pub fn set_repeat_params(&mut self, delay: u32, rate: u32) {
        self.repeat_delay = delay;
        self.repeat_rate = rate;
    }

    /// Convert keycode to keysym (basic mapping)
    fn keycode_to_keysym(&self, keycode: KeyCode) -> KeySym {
        // This is a very basic mapping - a real implementation would
        // use proper keyboard layouts and locale support
        match keycode {
            9 => 0xff1b,                               // Escape
            22 => 0xff08,                              // Backspace
            23 => 0xff09,                              // Tab
            36 => 0xff0d,                              // Return
            65 => 0x0020,                              // Space
            24..=33 => (keycode as u32 - 24) + 0x0071, // q-p
            38..=46 => (keycode as u32 - 38) + 0x0061, // a-l
            52..=58 => (keycode as u32 - 52) + 0x007a, // z-n
            10..=19 => {
                if keycode == 19 {
                    0x0030
                }
                // 0
                else {
                    (keycode as u32 - 10) + 0x0031
                } // 1-9
            }
            _ => keycode as u32, // Fallback
        }
    }
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self::new()
    }
}
