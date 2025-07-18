// display_system.rs
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::protocol::WindowId;
use crate::transport::TransportInfo;
use crate::{
    display::{
        config::DisplayConfig,
        create_display,
        types::{Display, DisplayTrait},
    },
    server::window_system::Window,
};

/// Manages display backends and rendering notifications
#[derive(Debug, Clone)]
pub struct DisplaySystem {
    displays: HashMap<TransportInfo, Arc<Mutex<Display>>>,
}

impl DisplaySystem {
    pub fn new() -> Self {
        Self {
            displays: HashMap::new(),
        }
    }

    /// Create displays from configuration
    pub fn from_configs(display_configs: Vec<DisplayConfig>) -> Result<Self> {
        let mut displays = HashMap::new();

        for config in display_configs {
            let transport_info = TransportInfo::new(config.transport, config.id);

            if displays.contains_key(&transport_info) {
                warn!(
                    "Duplicate display transport {:?} for '{}', skipping",
                    transport_info, config.name
                );
                continue;
            }

            let mut display = create_display(config)?;
            display.start()?;
            displays.insert(transport_info, Arc::new(Mutex::new(display)));
        }

        debug!("Set up {} display(s)", displays.len());
        Ok(Self { displays })
    }

    /// Get the number of displays
    pub fn display_count(&self) -> usize {
        self.displays.len()
    }

    /// Get displays reference
    pub fn displays(&self) -> &HashMap<TransportInfo, Arc<Mutex<Display>>> {
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
}

impl Default for DisplaySystem {
    fn default() -> Self {
        Self::new()
    }
}
