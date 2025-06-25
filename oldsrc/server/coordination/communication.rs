//! Inter-component communication
//!
//! Provides mechanisms for components to communicate with each other.

use crate::server::coordination::CoordinationResult;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Message types for inter-component communication
#[derive(Debug, Clone)]
pub enum ComponentMessage {
    /// Status update message
    StatusUpdate { component: String, status: String },
    /// Configuration change notification
    ConfigChange { component: String, config: String },
    /// Shutdown request
    Shutdown { component: String },
    /// Custom message
    Custom {
        from: String,
        to: String,
        data: Vec<u8>,
    },
}

/// Communication hub for managing inter-component messages
pub struct CommunicationHub {
    channels: HashMap<String, mpsc::UnboundedSender<ComponentMessage>>,
    broadcast_sender: mpsc::UnboundedSender<ComponentMessage>,
}

impl CommunicationHub {
    /// Create a new communication hub
    pub fn new() -> Self {
        let (broadcast_sender, _) = mpsc::unbounded_channel();
        Self {
            channels: HashMap::new(),
            broadcast_sender,
        }
    }

    /// Register a component for communication
    pub fn register_component(
        &mut self,
        component: String,
    ) -> mpsc::UnboundedReceiver<ComponentMessage> {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.channels.insert(component, sender);
        receiver
    }

    /// Send a message to a specific component
    pub fn send_to_component(
        &self,
        target: &str,
        message: ComponentMessage,
    ) -> CoordinationResult<()> {
        if let Some(sender) = self.channels.get(target) {
            sender.send(message).map_err(|e| {
                crate::server::coordination::CoordinationError::Communication(format!(
                    "Failed to send message to {}: {}",
                    target, e
                ))
            })?;
        }
        Ok(())
    }

    /// Broadcast a message to all components
    pub fn broadcast(&self, message: ComponentMessage) -> CoordinationResult<()> {
        for sender in self.channels.values() {
            sender.send(message.clone()).map_err(|e| {
                crate::server::coordination::CoordinationError::Communication(format!(
                    "Failed to broadcast message: {}",
                    e
                ))
            })?;
        }
        Ok(())
    }
}

impl Default for CommunicationHub {
    fn default() -> Self {
        Self::new()
    }
}
