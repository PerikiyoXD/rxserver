use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    display::{native_display::NativeDisplay, virtual_display::VirtualDisplay},
    protocol::WindowId,
    server::window_system::Window,
};

/// Messages for communicating with the virtual display thread
#[derive(Debug)]
pub enum DisplayMessage {
    UpdateFramebuffer(Vec<u32>),
    UpdateWindows(Vec<Window>), // Send all windows for re-rendering
    WindowCreated(Window),
    WindowMapped(WindowId),    // WindowId
    WindowUnmapped(WindowId),  // WindowId
    WindowDestroyed(WindowId), // WindowId
    Resize(u32, u32),
    Shutdown,
}

/// Messages sent from the virtual display back to the server
#[derive(Debug)]
pub enum DisplayCallbackMessage {
    WindowResized(u32, u32),
    DisplayClosed,
    /// Pointer moved to (x, y) in display-local pixel coordinates.
    PointerMoved(i32, i32),
    /// A pointer button was pressed. Button numbers follow X11 convention
    /// (1=left, 2=middle, 3=right, 4/5=wheel).
    PointerButtonPressed(u8),
    PointerButtonReleased(u8),
    /// A key was pressed/released, carrying winit's raw platform scancode -
    /// the server maps this to an X11 keycode (see `server::input`).
    KeyPressed(u32),
    KeyReleased(u32),
}

/// Display type enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayKind {
    Virtual,
    Native,
}

#[derive(Debug)]
pub enum Display {
    Virtual(VirtualDisplay),
    Native(NativeDisplay),
}

pub trait DisplayTrait {
    fn start(&mut self) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
    fn on_update_windows(&self, windows: Vec<Window>) -> Result<()>;
    fn on_window_created(&self, window: Window) -> Result<()>;
    fn on_window_mapped(&self, window_id: WindowId) -> Result<()>;
    fn on_window_unmapped(&self, window_id: WindowId) -> Result<()>;
    fn on_window_destroyed(&self, window_id: WindowId) -> Result<()>;
    /// Take ownership of this display's callback receiver, if it has one and
    /// it hasn't already been taken. Used once, right after `start()`, by
    /// whoever wants to consume `DisplayCallbackMessage`s (input events,
    /// resize, close) - see `Server::spawn_input_pump` in `state.rs`.
    fn take_callback_receiver(&mut self) -> Option<UnboundedReceiver<DisplayCallbackMessage>>;
}

impl DisplayTrait for Display {
    fn start(&mut self) -> Result<()> {
        match self {
            Display::Virtual(display) => display.start(),
            Display::Native(display) => display.start(),
        }
    }

    fn shutdown(&mut self) -> Result<()> {
        match self {
            Display::Virtual(display) => display.shutdown(),
            Display::Native(display) => display.shutdown(),
        }
    }

    fn on_update_windows(&self, windows: Vec<Window>) -> Result<()> {
        match self {
            Display::Virtual(display) => display.on_update_windows(windows),
            Display::Native(display) => display.on_update_windows(windows),
        }
    }

    fn on_window_created(&self, window: Window) -> Result<()> {
        match self {
            Display::Virtual(display) => display.on_window_created(window),
            Display::Native(display) => display.on_window_created(window),
        }
    }

    fn on_window_mapped(&self, window_id: WindowId) -> Result<()> {
        match self {
            Display::Virtual(display) => display.on_window_mapped(window_id),
            Display::Native(display) => display.on_window_mapped(window_id),
        }
    }

    fn on_window_unmapped(&self, window_id: WindowId) -> Result<()> {
        match self {
            Display::Virtual(display) => display.on_window_unmapped(window_id),
            Display::Native(display) => display.on_window_unmapped(window_id),
        }
    }

    fn on_window_destroyed(&self, window_id: WindowId) -> Result<()> {
        match self {
            Display::Virtual(display) => display.on_window_destroyed(window_id),
            Display::Native(display) => display.on_window_destroyed(window_id),
        }
    }

    fn take_callback_receiver(&mut self) -> Option<UnboundedReceiver<DisplayCallbackMessage>> {
        match self {
            Display::Virtual(display) => display.take_callback_receiver(),
            Display::Native(display) => display.take_callback_receiver(),
        }
    }
}
