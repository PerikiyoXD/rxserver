use std::sync::Arc;

use anyhow::Result;
use std::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::{error, info, warn};
#[cfg(target_os = "windows")]
use winit::{event_loop::EventLoop, platform::windows::EventLoopBuilderExtWindows};

use crate::{
    display::{
        config::DisplayConfig,
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

        // Clone the config for the thread
        let config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        // Spawn the display thread
        tokio::spawn(async move {
            info!("Starting virtual display thread");

            // Create event loop
            #[cfg(target_os = "windows")]
            let event_loop = EventLoop::builder()
                .with_any_thread(true)
                .build()
                .expect("Failed to create event loop");

            #[cfg(not(target_os = "windows"))]
            let event_loop = EventLoop::new().expect("Failed to create event loop");

            // Create application
            let mut app = VirtualDisplayApp::new(config, message_receiver, Some(callback_sender));

            // Run the event loop
            if let Err(e) = event_loop.run_app(&mut app) {
                error!("Virtual display event loop error: {}", e);
            }

            info!("Virtual display thread terminated");
        });

        info!("Virtual display started successfully");
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
