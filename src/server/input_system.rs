// input_system.rs
//! Backend-agnostic keyboard/pointer device model.
//!
//! `KeyboardDevice`/`PointerDevice` are the seam between real input sources
//! (a winit-driven virtual display today, real hardware - evdev/libinput on
//! Unix, raw input on Windows - later) and the server. Nothing above this
//! module (the input pump in `state.rs`, the eventual XInput handlers) is
//! allowed to know about winit or any other backend-specific type; they only
//! ever see `KeyEvent`/`PointerEvent`.

use async_trait::async_trait;

/// A key press/release, carrying a platform scancode. Translating that
/// scancode to an X11 keycode is the input pump's job (`state.rs`), not the
/// device's - a device only reports what physically happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub scancode: u32,
    pub pressed: bool,
}

/// A pointer motion or button event, in display-local pixel coordinates.
/// Button numbers follow X11 convention (1=left, 2=middle, 3=right,
/// 4/5=wheel).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerEvent {
    Moved { x: i32, y: i32 },
    Button { button: u8, pressed: bool },
}

/// A source of keyboard input. `next_event` resolves once when an event is
/// available - callers loop on it, same shape as reading from a channel.
#[async_trait]
pub trait KeyboardDevice: Send {
    async fn next_event(&mut self) -> Option<KeyEvent>;
}

/// A source of pointer input. Same polling shape as `KeyboardDevice`.
#[async_trait]
pub trait PointerDevice: Send {
    async fn next_event(&mut self) -> Option<PointerEvent>;
}
