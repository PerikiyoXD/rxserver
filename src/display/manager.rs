//! Runs every registered virtual display's window on a single winit
//! `EventLoop` owned by the real OS main thread. Displays are independent of
//! each other and of the server: closing one window (or all of them) does
//! not stop the others, and does not touch the async server thread at all.

use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use tracing::info;
use winit::{
    application::ApplicationHandler, event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
};

use crate::display::virtual_display_app::VirtualDisplayApp;

pub struct DisplayManager {
    pending: Vec<VirtualDisplayApp>,
    apps: HashMap<winit::window::WindowId, VirtualDisplayApp>,
    incoming: Receiver<VirtualDisplayApp>,
    ever_had_a_window: bool,
}

impl DisplayManager {
    pub fn new(incoming: Receiver<VirtualDisplayApp>) -> Self {
        Self {
            pending: Vec::new(),
            apps: HashMap::new(),
            incoming,
            ever_had_a_window: false,
        }
    }

    /// Pull in any displays registered since the last check (e.g. the server
    /// finished starting up after the event loop was already running).
    fn drain_incoming(&mut self, event_loop: &ActiveEventLoop) {
        while let Ok(app) = self.incoming.try_recv() {
            self.pending.push(app);
        }
        for mut app in self.pending.drain(..) {
            app.create_window(event_loop);
            if let Some(id) = app.window_id() {
                self.ever_had_a_window = true;
                self.apps.insert(id, app);
            }
        }
    }
}

impl ApplicationHandler for DisplayManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // The server registers its displays asynchronously on a different
        // thread, with no OS event to wake this loop back up. Poll instead
        // of the winit default (Wait) so `about_to_wait` keeps firing and
        // picks up newly-registered displays (and drains their message
        // queues) even with no window events happening.
        event_loop.set_control_flow(ControlFlow::Poll);
        self.drain_incoming(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(app) = self.apps.get_mut(&window_id) {
            app.handle_window_event(event_loop, event);
            if app.is_closed() {
                info!("Display window closed, removing from manager");
                self.apps.remove(&window_id);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.drain_incoming(event_loop);

        self.apps.retain(|_, app| {
            app.pump_messages();
            !app.is_closed()
        });

        // Exit the display event loop once every window that ever existed
        // has been closed. The server keeps running independently on its
        // own thread. Don't exit before any window has been created yet
        // (e.g. the server is still starting up).
        if self.ever_had_a_window && self.apps.is_empty() && self.pending.is_empty() {
            event_loop.exit();
        }
    }
}
