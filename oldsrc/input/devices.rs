//! Device management for input system

use super::types::{DeviceCapabilities, DeviceType, InputDevice};
use crate::types::Result;
use std::collections::HashMap;

/// Device manager for input devices
#[derive(Debug)]
pub struct DeviceManager {
    devices: HashMap<u32, InputDevice>,
    next_device_id: u32,
}

impl DeviceManager {
    /// Create new device manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            devices: HashMap::new(),
            next_device_id: 1,
        })
    }

    /// Scan for available input devices
    pub async fn scan_devices(&mut self) -> Result<()> {
        tracing::info!("Scanning for input devices");

        // TODO: Platform-specific device scanning
        // For now, create some mock devices
        self.add_mock_devices()?;

        tracing::info!("Found {} input devices", self.devices.len());
        Ok(())
    }

    /// Shutdown device manager
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down device manager");
        self.devices.clear();
        Ok(())
    }

    /// Get all devices
    pub fn devices(&self) -> &HashMap<u32, InputDevice> {
        &self.devices
    }

    /// Add mock devices for testing
    fn add_mock_devices(&mut self) -> Result<()> {
        // Mock keyboard
        let keyboard = InputDevice {
            id: self.next_device_id,
            name: "System Keyboard".to_string(),
            device_type: DeviceType::Keyboard,
            capabilities: DeviceCapabilities {
                has_buttons: false,
                has_axes: false,
                has_keys: true,
                button_count: 0,
                axis_count: 0,
            },
        };
        self.devices.insert(self.next_device_id, keyboard);
        self.next_device_id += 1;

        // Mock mouse
        let mouse = InputDevice {
            id: self.next_device_id,
            name: "System Mouse".to_string(),
            device_type: DeviceType::Mouse,
            capabilities: DeviceCapabilities {
                has_buttons: true,
                has_axes: true,
                has_keys: false,
                button_count: 3,
                axis_count: 2,
            },
        };
        self.devices.insert(self.next_device_id, mouse);
        self.next_device_id += 1;
        Ok(())
    }
}
