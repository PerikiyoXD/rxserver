//! Pointer Management System for X11 Server
//!
//! This module provides pointer input management, including pointer grabs,
//! event handling, and cursor tracking for the X11 server.

use crate::protocol::types::{Cursor, Window};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

/// Pointer grab modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabMode {
    Synchronous = 0,
    Asynchronous = 1,
}

impl From<u8> for GrabMode {
    fn from(value: u8) -> Self {
        match value {
            0 => GrabMode::Synchronous,
            1 => GrabMode::Asynchronous,
            _ => GrabMode::Asynchronous, // Default to asynchronous
        }
    }
}

/// Pointer grab status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabStatus {
    Success = 0,
    AlreadyGrabbed = 1,
    InvalidTime = 2,
    NotViewable = 3,
    Frozen = 4,
}

/// Information about an active pointer grab
#[derive(Debug, Clone)]
pub struct PointerGrab {
    /// Client that owns the grab
    pub client_id: u32,
    /// Window that requested the grab
    pub grab_window: Window,
    /// Whether owner events are reported
    pub owner_events: bool,
    /// Event mask for the grab
    pub event_mask: u16,
    /// Pointer mode (sync/async)
    pub pointer_mode: GrabMode,
    /// Keyboard mode (sync/async)
    pub keyboard_mode: GrabMode,
    /// Window to confine pointer to (0 = None)
    pub confine_to: Window,
    /// Cursor to display during grab (0 = None)
    pub cursor: Cursor,
    /// Timestamp when grab was established
    pub time: u32,
}

/// Thread-safe pointer manager for the X11 server
#[derive(Debug)]
pub struct PointerManager {
    /// Current pointer grab (if any)
    current_grab: Arc<RwLock<Option<PointerGrab>>>,
    /// Current pointer position
    pointer_position: Arc<RwLock<(i16, i16)>>,
    /// Current pointer window
    pointer_window: Arc<RwLock<Window>>,
    /// Button press state (button -> pressed)
    button_state: Arc<RwLock<HashMap<u8, bool>>>,
    /// Modifier key state
    modifier_state: Arc<RwLock<u16>>,
}

impl PointerManager {
    /// Create a new pointer manager
    pub fn new() -> Self {
        info!("Initializing PointerManager");

        PointerManager {
            current_grab: Arc::new(RwLock::new(None)),
            pointer_position: Arc::new(RwLock::new((0, 0))),
            pointer_window: Arc::new(RwLock::new(0)), // Root window
            button_state: Arc::new(RwLock::new(HashMap::new())),
            modifier_state: Arc::new(RwLock::new(0)),
        }
    }

    /// Attempt to grab the pointer
    pub fn grab_pointer(
        &self,
        client_id: u32,
        grab_window: Window,
        owner_events: bool,
        event_mask: u16,
        pointer_mode: GrabMode,
        keyboard_mode: GrabMode,
        confine_to: Window,
        cursor: Cursor,
        time: u32,
    ) -> GrabStatus {
        debug!(
            "Attempting pointer grab: client={}, window={}, owner_events={}, event_mask=0x{:04x}",
            client_id, grab_window, owner_events, event_mask
        );

        // Check if pointer is already grabbed
        {
            let current_grab = self.current_grab.read().unwrap();
            if current_grab.is_some() {
                warn!("Pointer grab failed: already grabbed by another client");
                return GrabStatus::AlreadyGrabbed;
            }
        }

        // TODO: Validate window exists and is viewable
        // For now, we'll assume all windows are valid and viewable

        // TODO: Validate timestamp
        // For now, we'll accept any timestamp

        // Create the grab
        let grab = PointerGrab {
            client_id,
            grab_window,
            owner_events,
            event_mask,
            pointer_mode,
            keyboard_mode,
            confine_to,
            cursor,
            time,
        };

        // Set the grab
        {
            let mut current_grab = self.current_grab.write().unwrap();
            *current_grab = Some(grab);
        }

        info!(
            "Pointer grab successful: client={}, window={}, mode={}",
            client_id,
            grab_window,
            match pointer_mode {
                GrabMode::Synchronous => "Sync",
                GrabMode::Asynchronous => "Async",
            }
        );

        GrabStatus::Success
    }

    /// Release the pointer grab
    pub fn ungrab_pointer(&self, client_id: u32) -> bool {
        debug!("Attempting to ungrab pointer: client={}", client_id);

        let ungrabbed = {
            let mut current_grab = self.current_grab.write().unwrap();
            match current_grab.as_ref() {
                Some(grab) if grab.client_id == client_id => {
                    *current_grab = None;
                    true
                }
                Some(_) => {
                    warn!("Ungrab failed: pointer grabbed by different client");
                    false
                }
                None => {
                    debug!("Ungrab ignored: pointer not grabbed");
                    false
                }
            }
        };

        if ungrabbed {
            info!("Pointer ungrabbed successfully: client={}", client_id);
        }

        ungrabbed
    }

    /// Get current pointer grab information
    pub fn get_current_grab(&self) -> Option<PointerGrab> {
        let current_grab = self.current_grab.read().unwrap();
        current_grab.clone()
    }

    /// Check if pointer is currently grabbed
    pub fn is_grabbed(&self) -> bool {
        let current_grab = self.current_grab.read().unwrap();
        current_grab.is_some()
    }

    /// Check if pointer is grabbed by a specific client
    pub fn is_grabbed_by_client(&self, client_id: u32) -> bool {
        let current_grab = self.current_grab.read().unwrap();
        match current_grab.as_ref() {
            Some(grab) => grab.client_id == client_id,
            None => false,
        }
    }

    /// Update pointer position
    pub fn set_pointer_position(&self, x: i16, y: i16) {
        let mut position = self.pointer_position.write().unwrap();
        *position = (x, y);
        debug!("Pointer position updated: ({}, {})", x, y);
    }

    /// Get current pointer position
    pub fn get_pointer_position(&self) -> (i16, i16) {
        let position = self.pointer_position.read().unwrap();
        *position
    }

    /// Update current pointer window
    pub fn set_pointer_window(&self, window: Window) {
        let mut pointer_window = self.pointer_window.write().unwrap();
        *pointer_window = window;
        debug!("Pointer window updated: {}", window);
    }

    /// Get current pointer window
    pub fn get_pointer_window(&self) -> Window {
        let pointer_window = self.pointer_window.read().unwrap();
        *pointer_window
    }

    /// Update button state
    pub fn set_button_state(&self, button: u8, pressed: bool) {
        let mut button_state = self.button_state.write().unwrap();
        button_state.insert(button, pressed);
        debug!(
            "Button {} state: {}",
            button,
            if pressed { "pressed" } else { "released" }
        );
    }

    /// Get button state
    pub fn get_button_state(&self, button: u8) -> bool {
        let button_state = self.button_state.read().unwrap();
        button_state.get(&button).copied().unwrap_or(false)
    }

    /// Update modifier state
    pub fn set_modifier_state(&self, modifiers: u16) {
        let mut modifier_state = self.modifier_state.write().unwrap();
        *modifier_state = modifiers;
        debug!("Modifier state updated: 0x{:04x}", modifiers);
    }

    /// Get current modifier state
    pub fn get_modifier_state(&self) -> u16 {
        let modifier_state = self.modifier_state.read().unwrap();
        *modifier_state
    }

    /// Force ungrab (used for cleanup when client disconnects)
    pub fn force_ungrab(&self, client_id: u32) {
        let ungrabbed = {
            let mut current_grab = self.current_grab.write().unwrap();
            match current_grab.as_ref() {
                Some(grab) if grab.client_id == client_id => {
                    *current_grab = None;
                    true
                }
                _ => false,
            }
        };

        if ungrabbed {
            info!("Forced pointer ungrab: client={}", client_id);
        }
    }

    /// Validate internal state consistency
    #[cfg(debug_assertions)]
    pub fn validate(&self) -> bool {
        // Basic validation - check that grab state is consistent
        let current_grab = self.current_grab.read().unwrap();
        if let Some(grab) = current_grab.as_ref() {
            if grab.grab_window == 0 {
                error!("Invalid grab: grab_window cannot be 0");
                return false;
            }
        }

        debug!("PointerManager validation passed");
        true
    }
}

impl Default for PointerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_manager_creation() {
        let manager = PointerManager::new();
        assert!(!manager.is_grabbed());
        assert_eq!(manager.get_pointer_position(), (0, 0));
        assert_eq!(manager.get_pointer_window(), 0);
    }

    #[test]
    fn test_grab_and_ungrab_pointer() {
        let manager = PointerManager::new();

        // Grab pointer
        let status = manager.grab_pointer(
            1,
            100,
            true,
            0x01FF,
            GrabMode::Asynchronous,
            GrabMode::Asynchronous,
            0,
            0,
            0,
        );
        assert_eq!(status, GrabStatus::Success);
        assert!(manager.is_grabbed());
        assert!(manager.is_grabbed_by_client(1));

        // Try to grab again (should fail)
        let status = manager.grab_pointer(
            2,
            101,
            false,
            0x01FF,
            GrabMode::Synchronous,
            GrabMode::Synchronous,
            0,
            0,
            0,
        );
        assert_eq!(status, GrabStatus::AlreadyGrabbed);

        // Ungrab pointer
        assert!(manager.ungrab_pointer(1));
        assert!(!manager.is_grabbed());

        // Try to ungrab again (should fail silently)
        assert!(!manager.ungrab_pointer(1));
    }

    #[test]
    fn test_grab_modes() {
        assert_eq!(GrabMode::from(0), GrabMode::Synchronous);
        assert_eq!(GrabMode::from(1), GrabMode::Asynchronous);
        assert_eq!(GrabMode::from(99), GrabMode::Asynchronous); // Default
    }

    #[test]
    fn test_pointer_position() {
        let manager = PointerManager::new();

        manager.set_pointer_position(100, 200);
        assert_eq!(manager.get_pointer_position(), (100, 200));

        manager.set_pointer_position(-50, -75);
        assert_eq!(manager.get_pointer_position(), (-50, -75));
    }

    #[test]
    fn test_button_state() {
        let manager = PointerManager::new();

        // Initially no buttons pressed
        assert!(!manager.get_button_state(1));
        assert!(!manager.get_button_state(2));

        // Press button 1
        manager.set_button_state(1, true);
        assert!(manager.get_button_state(1));
        assert!(!manager.get_button_state(2));

        // Release button 1, press button 2
        manager.set_button_state(1, false);
        manager.set_button_state(2, true);
        assert!(!manager.get_button_state(1));
        assert!(manager.get_button_state(2));
    }

    #[test]
    fn test_force_ungrab() {
        let manager = PointerManager::new();

        // Grab pointer
        let status = manager.grab_pointer(
            1,
            100,
            true,
            0x01FF,
            GrabMode::Asynchronous,
            GrabMode::Asynchronous,
            0,
            0,
            0,
        );
        assert_eq!(status, GrabStatus::Success);
        assert!(manager.is_grabbed_by_client(1));

        // Force ungrab
        manager.force_ungrab(1);
        assert!(!manager.is_grabbed());

        // Force ungrab non-existent grab (should do nothing)
        manager.force_ungrab(2);
        assert!(!manager.is_grabbed());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_pointer_manager_validation() {
        let manager = PointerManager::new();

        // Initially should be valid
        assert!(manager.validate());

        // After grabbing should still be valid
        let status = manager.grab_pointer(
            1,
            100,
            true,
            0x01FF,
            GrabMode::Asynchronous,
            GrabMode::Asynchronous,
            0,
            0,
            0,
        );
        assert_eq!(status, GrabStatus::Success);
        assert!(manager.validate());
    }
}
