//! X11 Event System
//!
//! This module implements the X11 event generation, queuing, and delivery system.
//! Events are generated in response to state changes and delivered to interested clients.

use crate::types::Result;

/// X11 Event types
#[derive(Debug, Clone)]
pub enum X11Event {
    /// Key press event
    KeyPress {
        window: u32,
        key: u8,
        state: u16,
        time: u32,
        x: i16,
        y: i16,
    },
    /// Key release event
    KeyRelease {
        window: u32,
        key: u8,
        state: u16,
        time: u32,
        x: i16,
        y: i16,
    },
    /// Button press event
    ButtonPress {
        window: u32,
        button: u8,
        state: u16,
        time: u32,
        x: i16,
        y: i16,
    },
    /// Button release event
    ButtonRelease {
        window: u32,
        button: u8,
        state: u16,
        time: u32,
        x: i16,
        y: i16,
    },
    /// Expose event
    Expose {
        window: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        count: u16,
    },
    /// Configure notify event
    ConfigureNotify {
        window: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
    },
}

/// Event dispatcher interface
pub trait EventDispatcher {
    /// Send an event to a client
    fn send_event(&mut self, client_id: u32, event: X11Event) -> Result<()>;

    /// Queue an event for delivery
    fn queue_event(&mut self, client_id: u32, event: X11Event) -> Result<()>;

    /// Flush queued events
    fn flush_events(&mut self) -> Result<()>;
}

/// Basic event dispatcher implementation
#[derive(Debug, Default)]
pub struct BasicEventDispatcher {
    /// Event queue
    queue: Vec<(u32, X11Event)>,
}

impl BasicEventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Get the number of queued events
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }
}

impl EventDispatcher for BasicEventDispatcher {
    fn send_event(&mut self, client_id: u32, event: X11Event) -> Result<()> {
        // For now, just queue the event
        self.queue_event(client_id, event)
    }

    fn queue_event(&mut self, client_id: u32, event: X11Event) -> Result<()> {
        self.queue.push((client_id, event));
        Ok(())
    }

    fn flush_events(&mut self) -> Result<()> {
        // For now, just clear the queue
        // In a real implementation, this would send events to clients
        self.queue.clear();
        Ok(())
    }
}

/// Event generation utilities
pub mod generator {
    use super::*;

    /// Generate an expose event for a window
    pub fn generate_expose_event(window: u32, x: i16, y: i16, width: u16, height: u16) -> X11Event {
        X11Event::Expose {
            window,
            x,
            y,
            width,
            height,
            count: 0,
        }
    }

    /// Generate a configure notify event
    pub fn generate_configure_notify(
        window: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> X11Event {
        X11Event::ConfigureNotify {
            window,
            x,
            y,
            width,
            height,
            border_width: 0,
        }
    }
}
