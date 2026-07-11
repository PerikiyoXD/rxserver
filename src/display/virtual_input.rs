//! Winit-backed `KeyboardDevice`/`PointerDevice` implementations.
//!
//! `VirtualDisplayApp` reports raw winit input as `DisplayCallbackMessage`s
//! on one channel (see `virtual_display_app.rs`). `spawn_demux` reads that
//! channel once and fans it out into a keyboard stream and a pointer stream,
//! each wrapped as a `KeyboardDevice`/`PointerDevice` - the only two types
//! anything above this module (the server's input pump, XInput handlers)
//! should ever see. A future hardware backend (evdev/libinput on Unix, raw
//! input on Windows) implements the same two traits directly, with no
//! `DisplayCallbackMessage` involved at all.

use async_trait::async_trait;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::warn;

use crate::{
    display::types::DisplayCallbackMessage,
    server::input_system::{KeyEvent, KeyboardDevice, PointerDevice, PointerEvent},
};

/// Reads `callback_receiver` to completion, forwarding keyboard messages to
/// `keyboard_tx` and pointer messages to `pointer_tx`. Runs as a background
/// task for the lifetime of the display; exits when the channel closes
/// (display shut down).
pub fn spawn_demux(
    mut callback_receiver: UnboundedReceiver<DisplayCallbackMessage>,
) -> (VirtualKeyboardDevice, VirtualPointerDevice) {
    let (keyboard_tx, keyboard_rx) = unbounded_channel();
    let (pointer_tx, pointer_rx) = unbounded_channel();

    tokio::spawn(async move {
        while let Some(message) = callback_receiver.recv().await {
            forward(message, &keyboard_tx, &pointer_tx);
        }
    });

    (
        VirtualKeyboardDevice { rx: keyboard_rx },
        VirtualPointerDevice { rx: pointer_rx },
    )
}

fn forward(
    message: DisplayCallbackMessage,
    keyboard_tx: &UnboundedSender<KeyEvent>,
    pointer_tx: &UnboundedSender<PointerEvent>,
) {
    match message {
        DisplayCallbackMessage::KeyPressed(scancode) => {
            let _ = keyboard_tx.send(KeyEvent {
                scancode,
                pressed: true,
            });
        }
        DisplayCallbackMessage::KeyReleased(scancode) => {
            let _ = keyboard_tx.send(KeyEvent {
                scancode,
                pressed: false,
            });
        }
        DisplayCallbackMessage::PointerMoved(x, y) => {
            let _ = pointer_tx.send(PointerEvent::Moved { x, y });
        }
        DisplayCallbackMessage::PointerButtonPressed(button) => {
            let _ = pointer_tx.send(PointerEvent::Button {
                button,
                pressed: true,
            });
        }
        DisplayCallbackMessage::PointerButtonReleased(button) => {
            let _ = pointer_tx.send(PointerEvent::Button {
                button,
                pressed: false,
            });
        }
        DisplayCallbackMessage::WindowResized(_, _) | DisplayCallbackMessage::DisplayClosed => {
            // Not input - not this demux's concern. Logged rather than
            // silently dropped since nothing else currently consumes these
            // either (see investigate_xinput_disconnect handoff notes).
            warn!(
                "VirtualDisplay callback {:?} has no consumer yet",
                message
            );
        }
    }
}

pub struct VirtualKeyboardDevice {
    rx: UnboundedReceiver<KeyEvent>,
}

impl std::fmt::Debug for VirtualKeyboardDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualKeyboardDevice").finish_non_exhaustive()
    }
}

#[async_trait]
impl KeyboardDevice for VirtualKeyboardDevice {
    async fn next_event(&mut self) -> Option<KeyEvent> {
        self.rx.recv().await
    }
}

pub struct VirtualPointerDevice {
    rx: UnboundedReceiver<PointerEvent>,
}

impl std::fmt::Debug for VirtualPointerDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualPointerDevice").finish_non_exhaustive()
    }
}

#[async_trait]
impl PointerDevice for VirtualPointerDevice {
    async fn next_event(&mut self) -> Option<PointerEvent> {
        self.rx.recv().await
    }
}
