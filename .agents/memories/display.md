# display

`src/display/`. Renders windows to a virtual display (a real OS
window, via `softbuffer`).

## Thread ownership: this matters, do not "simplify" it

winit's event loop must run on the real OS main thread. The async
server (tokio, all the request handling) runs on its own thread with
its own runtime. `main.rs` owns both:

- Real main thread: `EventLoop` + `DisplayManager`
  (`display::manager`), which can host multiple display windows.
- Spawned thread: tokio runtime running `RX11Server::run()`.

`VirtualDisplay::start()` does not spawn its own event loop - it
registers a `VirtualDisplayApp` via `display::registry` (a channel)
for `main.rs` to pick up and run. This decoupling exists because
running winit off the main thread (or tying its lifetime to the
server) previously meant closing the display window took the whole
server process down with it. Closing the display window must never
stop the server, and the server dying must never be required to close
the window.

Winit defaults to `ControlFlow::Wait`; this codebase sets `Poll` in
`DisplayManager::resumed()` because the server registers its display
asynchronously with no OS event to wake the loop back up otherwise.

## Pixel format: two different formats, don't mix them up

- `softbuffer`'s framebuffer (`VirtualDisplayApp::framebuffer`):
  `0x00RRGGBB`. No alpha byte. Top byte must be zero.
- `Window::pixel_data` (server-side, the window's own content):
  `0xAARRGGBB`. Real alpha channel, for future compositing.

`draw_window_content` masks off the alpha byte
(`pixel & 0x00FF_FFFF`) when blitting from a window into the
framebuffer. If you add a new place that writes into the framebuffer
directly, use the `0x00RRGGBB` format or colors will be shifted by a
byte and look wrong (this happened once - every color constant was
off by exactly one byte).

## Debug overlay

`DisplayConfig::debug_overlay` (rxserver.toml) gates a dev-only
overlay (window count/status dots) drawn on top of the real render.
Off by default. Not real X11 output, just internal state
visualization.
