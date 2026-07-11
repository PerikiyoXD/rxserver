//! Handoff point between the async server (which owns display configuration
//! and lifecycle) and the real OS main thread (which must own winit's event
//! loop). `VirtualDisplay::start` cannot run its own event loop on a tokio
//! task without risking taking the whole process down when a window closes
//! (winit's Win32 backend expects to own the thread it runs on), so instead
//! it registers a pending `VirtualDisplayApp` here for `main.rs` to collect
//! and run on the main thread.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, OnceLock};

use crate::display::virtual_display_app::VirtualDisplayApp;

static REGISTRY: OnceLock<Mutex<Sender<VirtualDisplayApp>>> = OnceLock::new();
static RECEIVER: OnceLock<Mutex<Option<Receiver<VirtualDisplayApp>>>> = OnceLock::new();

fn init() -> &'static Mutex<Sender<VirtualDisplayApp>> {
    REGISTRY.get_or_init(|| {
        let (tx, rx) = channel();
        RECEIVER.get_or_init(|| Mutex::new(Some(rx)));
        Mutex::new(tx)
    })
}

/// Register a display app to be run on the main thread's event loop.
pub fn register(app: VirtualDisplayApp) {
    let sender = init().lock().expect("display registry sender poisoned");
    let _ = sender.send(app);
}

/// Take the receiving end of the registry. Must be called from `main.rs`
/// before the server starts registering displays; returns `None` if already
/// taken.
pub fn take_receiver() -> Option<Receiver<VirtualDisplayApp>> {
    init();
    RECEIVER
        .get()
        .expect("receiver initialized by init()")
        .lock()
        .expect("display registry receiver poisoned")
        .take()
}
