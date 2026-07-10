use std::sync::Arc;

use anyhow::Result;
use std::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::{info, warn};

use crate::{
    display::{
        config::DisplayConfig,
        registry,
        types::{DisplayCallbackMessage, DisplayMessage, DisplayTrait},
        virtual_display_app::VirtualDisplayApp,
    },
    server::window_system::Window,
};

/// Virtual display manager that handles the display thread
#[derive(Debug)]
pub struct VirtualDisplay {
    config: Arc<Mutex<DisplayConfig>>,
    message_sender: Option<UnboundedSender<DisplayMessage>>,
    callback_receiver: Option<UnboundedReceiver<DisplayCallbackMessage>>,
}

impl VirtualDisplay {
    pub fn new(config: DisplayConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            message_sender: None,
            callback_receiver: None,
        }
    }
}

impl Drop for VirtualDisplay {
    fn drop(&mut self) {
        if let Err(e) = self.shutdown() {
            warn!("Error shutting down virtual display: {}", e);
        }
    }
}

impl DisplayTrait for VirtualDisplay {
    fn start(&mut self) -> Result<()> {
        // Create RX TX channels for communication
        let (message_sender, message_receiver) = unbounded_channel();
        let (callback_sender, callback_receiver) = unbounded_channel();

        // Store the channels
        self.message_sender = Some(message_sender);
        self.callback_receiver = Some(callback_receiver);

        // Clone the config for the app
        let config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        // winit's event loop must run on the real OS main thread (its Win32
        // backend in particular does not tolerate being driven from a
        // background thread/task, and closing such a window can take the
        // whole process down with it). Register the app for main.rs to pick
        // up and run instead of starting our own event loop here, so the
        // display's lifetime stays independent of the server's.
        let app = VirtualDisplayApp::new(config, message_receiver, Some(callback_sender));
        registry::register(app);

        info!("Virtual display registered for main-thread event loop");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::Shutdown)
                .map_err(|e| anyhow::anyhow!("Failed to send shutdown: {}", e))?;
        }
        Ok(())
    }

    fn on_update_windows(&self, windows: Vec<Window>) -> Result<()> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::UpdateWindows(windows))
                .map_err(|e| anyhow::anyhow!("Failed to send window update: {}", e))?;
        }
        Ok(())
    }

    fn on_window_created(&self, window: Window) -> Result<()> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::WindowCreated(window))
                .map_err(|e| anyhow::anyhow!("Failed to send window creation: {}", e))?;
        }
        Ok(())
    }

    fn on_window_mapped(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::WindowMapped(window_id))
                .map_err(|e| anyhow::anyhow!("Failed to send window mapping: {}", e))?;
        }
        Ok(())
    }

    fn on_window_unmapped(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::WindowUnmapped(window_id))
                .map_err(|e| anyhow::anyhow!("Failed to send window unmapping: {}", e))?;
        }
        Ok(())
    }

    fn on_window_destroyed(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::WindowDestroyed(window_id))
                .map_err(|e| anyhow::anyhow!("Failed to send window destruction: {}", e))?;
        }
        Ok(())
    }
}
