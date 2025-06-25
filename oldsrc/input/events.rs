//! Event processing for input system

use super::types::InputEvent;
use crate::types::Result;
use std::collections::VecDeque;
use tokio::sync::mpsc;

/// Event processor for input events
#[derive(Debug)]
pub struct EventProcessor {
    event_queue: VecDeque<InputEvent>,
    event_receiver: Option<mpsc::UnboundedReceiver<InputEvent>>,
    is_running: bool,
}

impl EventProcessor {
    /// Create new event processor
    pub fn new() -> Result<Self> {
        Ok(Self {
            event_queue: VecDeque::new(),
            event_receiver: None,
            is_running: false,
        })
    }

    /// Start the event processor
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting input event processor");
        self.is_running = true;
        Ok(())
    }

    /// Stop the event processor
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping input event processor");
        self.is_running = false;
        self.event_queue.clear();
        Ok(())
    }

    /// Process queued events
    pub fn process_events(&mut self) -> Vec<InputEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.event_queue.pop_front() {
            events.push(event);
        }
        events
    }

    /// Add event to queue
    pub fn queue_event(&mut self, event: InputEvent) {
        self.event_queue.push_back(event);
    }
}

/// Event queue for batching input events
#[derive(Debug)]
pub struct EventQueue {
    events: VecDeque<InputEvent>,
    max_size: usize,
}

impl EventQueue {
    /// Create new event queue
    pub fn new(max_size: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_size,
        }
    }

    /// Add event to queue
    pub fn push(&mut self, event: InputEvent) {
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Get all events and clear queue
    pub fn drain(&mut self) -> Vec<InputEvent> {
        self.events.drain(..).collect()
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
