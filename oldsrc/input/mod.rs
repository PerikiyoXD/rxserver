//! Input management system
//!
//! This module provides input device detection, management, and event processing
//! for the X11 server.

pub mod devices;
pub mod events;
pub mod manager;
pub mod types;

// Re-export commonly used types
pub use devices::DeviceManager;
pub use events::{EventProcessor, EventQueue};
pub use manager::InputManager;
pub use types::{DeviceCapabilities, DeviceType, InputConfiguration, InputDevice, InputEvent};

use crate::types::Result;
use std::sync::Arc;

/// Input system coordinator
#[derive(Debug)]
pub struct InputSystem {
    manager: InputManager,
    device_manager: DeviceManager,
    event_processor: EventProcessor,
}

impl InputSystem {
    /// Create a new input system
    pub fn new(config: InputConfiguration) -> Result<Self> {
        Ok(Self {
            manager: InputManager::new(config)?,
            device_manager: DeviceManager::new()?,
            event_processor: EventProcessor::new()?,
        })
    }

    /// Initialize the input system
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing input system");

        self.device_manager.scan_devices().await?;
        self.manager.initialize().await?;
        self.event_processor.start().await?;

        tracing::info!("Input system initialized successfully");
        Ok(())
    }

    /// Shutdown the input system
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down input system");

        self.event_processor.stop().await?;
        self.manager.shutdown().await?;
        self.device_manager.shutdown().await?;

        Ok(())
    }

    /// Get input manager reference
    pub fn manager(&self) -> &InputManager {
        &self.manager
    }

    /// Get device manager reference  
    pub fn device_manager(&self) -> &DeviceManager {
        &self.device_manager
    }

    /// Get event processor reference
    pub fn event_processor(&self) -> &EventProcessor {
        &self.event_processor
    }
}
