// main.rs
use anyhow::{Context, Result};
use rxserver::{
    display::{manager::DisplayManager, registry},
    logging::init_logging,
    server::{RX11Server, config::load_config},
};
use tracing::error;
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    // The display event loop must run on the real OS main thread (winit's
    // Win32 backend in particular does not tolerate being driven from a
    // background thread, and closing such a window could take the whole
    // process down with it). The async server runs independently on its own
    // thread with its own tokio runtime, so a display window closing never
    // affects the server, and vice versa.
    let display_receiver =
        registry::take_receiver().expect("display registry receiver already taken");

    let server_thread = std::thread::Builder::new()
        .name("rxserver-async".to_string())
        .spawn(run_server)
        .context("Failed to spawn server thread")?;

    let event_loop = EventLoop::new().context("Failed to create display event loop")?;
    let mut manager = DisplayManager::new(display_receiver);
    if let Err(e) = event_loop.run_app(&mut manager) {
        error!("Display event loop error: {}", e);
    }

    // The display event loop exiting (all windows closed) does not stop the
    // server: keep the process alive by waiting on the server thread, so
    // rxserver keeps serving headless with no window open. The server only
    // stops on Ctrl+C or a fatal error of its own.
    let _ = server_thread.join();

    Ok(())
}

fn run_server() {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("rxserver-worker")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            return;
        }
    };

    if let Err(e) = rt.block_on(async_main()) {
        error!("Server error: {}", e);
    }
}

async fn async_main() -> Result<()> {
    let config = load_config(None).context("Failed to load server configuration")?;
    let logging = config.logging.clone();
    init_logging(logging).context("Failed to initialize logging")?;

    let server = RX11Server::new(config).context("Failed to create X11 server")?;
    server.run().await.context("Failed to run X11 server")?;

    Ok(())
}
