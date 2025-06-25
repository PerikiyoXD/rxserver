//! Input manager implementation

use super::types::{InputConfiguration, InputDevice, InputEvent};
use crate::types::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Main input manager
#[derive(Debug)]
pub struct InputManager {
    config: InputConfiguration,
    devices: HashMap<u32, InputDevice>,
    event_sender: Option<mpsc::UnboundedSender<InputEvent>>,
}

impl InputManager {
    /// Create new input manager
    pub fn new(config: InputConfiguration) -> Result<Self> {
        Ok(Self {
            config,
            devices: HashMap::new(),
            event_sender: None,
        })
    }

    /// Initialize the input manager
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing input manager");

        // Create event channel
        let (sender, _receiver) = mpsc::unbounded_channel();
        self.event_sender = Some(sender);

        tracing::info!("Input manager initialized");
        Ok(())
    }

    /// Shutdown the input manager
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down input manager");

        self.devices.clear();
        self.event_sender = None;

        Ok(())
    }

    /// Register an input device
    pub fn register_device(&mut self, device: InputDevice) -> Result<()> {
        let device_id = device.id;
        self.devices.insert(device_id, device);
        tracing::info!("Registered input device: {}", device_id);
        Ok(())
    }

    /// Get input configuration
    pub fn config(&self) -> &InputConfiguration {
        &self.config
    }

    /// Get registered devices
    pub fn devices(&self) -> &HashMap<u32, InputDevice> {
        &self.devices
    }
}
