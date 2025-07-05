use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tracing::info;

use crate::display::{
    config::DisplayConfig,
    types::{DisplayCallbackMessage, DisplayMessage, DisplayTrait},
};

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
        let (message_sender, _message_receiver): (
            UnboundedSender<DisplayMessage>,
            UnboundedReceiver<DisplayMessage>,
        ) = unbounded_channel();
        let (_callback_sender, callback_receiver): (
            UnboundedSender<DisplayCallbackMessage>,
            UnboundedReceiver<DisplayCallbackMessage>,
        ) = unbounded_channel();

        // Store the channels
        self.message_sender = Some(message_sender);
        self.callback_receiver = Some(callback_receiver);

        // Spawn the display thread
        std::thread::spawn(move || {
            info!("Starting native display thread");
            todo!("Implement native display thread logic");
        });

        Ok(())
    }
}

impl DisplayTrait for NativeDisplay {
    fn start(&mut self) -> Result<()> {
        todo!()
    }

    fn shutdown(&mut self) -> Result<()> {
        todo!()
    }

    fn on_update_windows(&self, windows: Vec<crate::server::WindowState>) -> Result<()> {
        todo!()
    }

    fn on_window_created(&self, window: crate::server::WindowState) -> Result<()> {
        todo!()
    }

    fn on_window_mapped(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        todo!()
    }

    fn on_window_unmapped(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        todo!()
    }

    fn on_window_destroyed(&self, window_id: crate::protocol::WindowId) -> Result<()> {
        todo!()
    }
}

pub struct NativeDisplayApp {
    config: Arc<Mutex<DisplayConfig>>,
    message_receiver: UnboundedReceiver<DisplayMessage>,
    callback_sender: Option<UnboundedSender<DisplayCallbackMessage>>,
}

impl NativeDisplayApp {
    pub fn new(
        config: Arc<Mutex<DisplayConfig>>,
        message_receiver: UnboundedReceiver<DisplayMessage>,
        callback_sender: Option<UnboundedSender<DisplayCallbackMessage>>,
    ) -> Self {
        Self {
            config,
            message_receiver,
            callback_sender,
        }
    }

    // pub fn run(self, event_loop: EventLoop<Self>) -> Result<()> {
    //     // Implement the logic to run the native display application
    //     // This will include handling events, rendering content, etc.
    //     Ok(())
    // }
}
