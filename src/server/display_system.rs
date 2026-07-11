// display_system.rs
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::protocol::WindowId;
use crate::{
    display::{
        config::DisplayConfig,
        create_display,
        types::{Display, DisplayTrait},
        virtual_input::{VirtualKeyboardDevice, VirtualPointerDevice, spawn_demux},
    },
    protocol::randr::RandrState,
    server::window_system::Window,
};

/// Manages display backends and rendering notifications
#[derive(Debug)]
pub struct DisplaySystem {
    displays: HashMap<usize, Arc<Mutex<Display>>>,
    randr_state: RandrState,
    /// The server's core keyboard/pointer devices. X11 has one core pointer
    /// and one core keyboard per server (not per screen), so these come from
    /// whichever display started first - not one pair per display. `None`
    /// until a display actually produces a callback receiver to demux (the
    /// `Native` backend is still a stub and never will).
    keyboard_device: Option<VirtualKeyboardDevice>,
    pointer_device: Option<VirtualPointerDevice>,
}

impl DisplaySystem {
    pub fn new() -> Self {
        Self {
            displays: HashMap::new(),
            randr_state: RandrState::new(),
            keyboard_device: None,
            pointer_device: None,
        }
    }

    /// Create displays from configuration
    pub fn from_configs(display_configs: Vec<DisplayConfig>) -> Result<Self> {
        let mut displays = HashMap::new();
        let mut randr_state = RandrState::new();
        let mut keyboard_device = None;
        let mut pointer_device = None;

        for config in display_configs {
            let display_id = config.id;

            if displays.contains_key(&display_id) {
                warn!(
                    "Duplicate display ID {} for '{}', skipping",
                    display_id, config.name
                );
                continue;
            }

            let mut display = create_display(config.clone())?;
            display.start()?;

            if keyboard_device.is_none() {
                if let Some(callback_receiver) = display.take_callback_receiver() {
                    let (keyboard, pointer) = spawn_demux(callback_receiver);
                    keyboard_device = Some(keyboard);
                    pointer_device = Some(pointer);
                }
            }

            // Initialize RANDR state for this screen
            randr_state.init_screen(
                display_id as u32,
                config.resolution[0] as u16,
                config.resolution[1] as u16,
            );

            displays.insert(display_id, Arc::new(Mutex::new(display)));
        }

        debug!("Set up {} display(s)", displays.len());
        Ok(Self {
            displays,
            randr_state,
            keyboard_device,
            pointer_device,
        })
    }

    /// Take the server-wide keyboard device, if one was produced by a
    /// display backend. Consumed once, by whoever spawns the input pump
    /// (`RX11Server::run` in `mod.rs`).
    pub fn take_keyboard_device(&mut self) -> Option<VirtualKeyboardDevice> {
        self.keyboard_device.take()
    }

    /// Take the server-wide pointer device. See `take_keyboard_device`.
    pub fn take_pointer_device(&mut self) -> Option<VirtualPointerDevice> {
        self.pointer_device.take()
    }

    /// Get the number of displays
    pub fn display_count(&self) -> usize {
        self.displays.len()
    }

    /// Get displays reference
    pub fn displays(&self) -> &HashMap<usize, Arc<Mutex<Display>>> {
        &self.displays
    }

    /// Shutdown all displays
    pub async fn shutdown(&self) {
        for display in self.displays.values() {
            let mut display_guard = display.lock().await;
            if let Err(e) = display_guard.shutdown() {
                warn!("Error shutting down display: {}", e);
            }
        }
    }

    /// Notify all displays of window creation
    pub async fn notify_window_created(&self, window: &Window) {
        for display in self.displays.values() {
            let display_guard = display.lock().await;
            if let Err(e) = display_guard.on_window_created(window.clone()) {
                debug!("Failed to notify display of window creation: {}", e);
            }
        }
    }

    /// Notify all displays of window mapping
    pub async fn notify_window_mapped(&self, window_id: WindowId) {
        for display in self.displays.values() {
            let display_guard = display.lock().await;
            if let Err(e) = display_guard.on_window_mapped(window_id) {
                debug!("Failed to notify display of window mapping: {}", e);
            }
        }
    }

    /// Notify all displays of window unmapping
    pub async fn notify_window_unmapped(&self, window_id: WindowId) {
        for display in self.displays.values() {
            let display_guard = display.lock().await;
            if let Err(e) = display_guard.on_window_unmapped(window_id) {
                debug!("Failed to notify display of window unmapping: {}", e);
            }
        }
    }

    /// Notify all displays of window destruction
    pub async fn notify_window_destroyed(&self, window_id: WindowId) {
        for display in self.displays.values() {
            let display_guard = display.lock().await;
            if let Err(e) = display_guard.on_window_destroyed(window_id) {
                debug!("Failed to notify display of window destruction: {}", e);
            }
        }
    }

    /// Send current window state to all displays
    pub async fn sync_windows(&self, windows: Vec<Window>) {
        for display in self.displays.values() {
            let display_guard = display.lock().await;
            if let Err(e) = display_guard.on_update_windows(windows.clone()) {
                debug!("Failed to sync windows to display: {}", e);
            }
        }
    }

    /// Get RANDR state
    pub fn randr_state(&self) -> &RandrState {
        &self.randr_state
    }

    /// Get mutable RANDR state
    pub fn randr_state_mut(&mut self) -> &mut RandrState {
        &mut self.randr_state
    }
}

impl Default for DisplaySystem {
    fn default() -> Self {
        Self::new()
    }
}
