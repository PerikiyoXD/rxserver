use crate::{
    display::{
        config::DisplayConfig,
        types::{DisplayCallbackMessage, DisplayMessage, DisplayTrait},
    },
    server::window_system::Window,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tracing::info;

#[derive(Debug)]
pub struct NativeDisplay {
    config: Arc<Mutex<DisplayConfig>>,
    message_sender: Option<UnboundedSender<DisplayMessage>>,
    callback_receiver: Option<UnboundedReceiver<DisplayCallbackMessage>>,
}

impl NativeDisplay {
    /// Create a new native display with the given configuration
    pub fn new(config: DisplayConfig) -> Self {
        let config = Arc::new(Mutex::new(config));
        Self {
            config,
            message_sender: None,
            callback_receiver: None,
        }
    }

    /// Start the native display in a separate thread
    pub fn start(&mut self) -> Result<()> {
        let (message_sender, message_receiver): (
            UnboundedSender<DisplayMessage>,
            UnboundedReceiver<DisplayMessage>,
        ) = unbounded_channel();

        let (callback_sender, callback_receiver): (
            UnboundedSender<DisplayCallbackMessage>,
            UnboundedReceiver<DisplayCallbackMessage>,
        ) = unbounded_channel();

        // Store the channels
        self.message_sender = Some(message_sender);
        self.callback_receiver = Some(callback_receiver);

        // Clone config for the thread
        let config = Arc::clone(&self.config);

        // Spawn the display thread
        std::thread::spawn(move || {
            info!("Starting native display thread");
            let _app = NativeDisplayApp::new(config, message_receiver, Some(callback_sender));
            // TODO: Run the display app
            // app.run();
        });

        Ok(())
    }
}

impl DisplayTrait for NativeDisplay {
    fn start(&mut self) -> Result<()> {
        self.start()
    }

    fn shutdown(&mut self) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(DisplayMessage::Shutdown);
        }
        Ok(())
    }

    fn on_update_windows(&self, windows: Vec<Window>) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(DisplayMessage::UpdateWindows(windows));
        }
        Ok(())
    }

    fn on_window_created(&self, window: Window) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(DisplayMessage::WindowCreated(window));
        }
        Ok(())
    }

    fn on_window_mapped(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(DisplayMessage::WindowMapped(window_id));
        }
        Ok(())
    }

    fn on_window_unmapped(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(DisplayMessage::WindowUnmapped(window_id));
        }
        Ok(())
    }

    fn on_window_destroyed(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(DisplayMessage::WindowDestroyed(window_id));
        }
        Ok(())
    }
}

pub struct NativeDisplayApp {
    _config: Arc<Mutex<DisplayConfig>>,
    _message_receiver: UnboundedReceiver<DisplayMessage>,
    _callback_sender: Option<UnboundedSender<DisplayCallbackMessage>>,
}

impl NativeDisplayApp {
    pub fn new(
        config: Arc<Mutex<DisplayConfig>>,
        message_receiver: UnboundedReceiver<DisplayMessage>,
        callback_sender: Option<UnboundedSender<DisplayCallbackMessage>>,
    ) -> Self {
        Self {
            _config: config,
            _message_receiver: message_receiver,
            _callback_sender: callback_sender,
        }
    }

    // pub fn run(self, event_loop: EventLoop<Self>) -> Result<()> {
    //     // Implement the logic to run the native display application
    //     // This will include handling events, rendering content, etc.
    //     Ok(())
    // }
}
