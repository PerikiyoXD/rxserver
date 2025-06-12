//! Simple test for the window renderer
//!
//! This example creates a window and displays an animated pattern to verify
//! that the ScreenWindowRenderer is working properly.

use rxserver::{
    display::{
        framebuffer::{Framebuffer, FramebufferConfig, PixelFormat},
        window_renderer::{ScreenWindowRenderer, WindowRendererConfig},
    },
    Result,
};
use std::{sync::Arc, time::Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rxserver=info,test_window_renderer=debug")
        .with_target(true)
        .init();

    println!("🚀 Starting Window Renderer Test");

    // Create event loop (must be on main thread)
    let event_loop = EventLoop::new()
        .map_err(|e| rxserver::Error::Display(format!("Failed to create event loop: {}", e)))?;

    // Create window
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("RX Server - Window Renderer Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .with_resizable(true)
            .build(&event_loop)
            .map_err(|e| rxserver::Error::Display(format!("Failed to create window: {}", e)))?,
    );

    // Create renderer config
    let config = WindowRendererConfig {
        title: "RX Server - Window Renderer Test".to_string(),
        width: 800,
        height: 600,
        vsync: true,
        target_fps: 60,
    };

    // Create screen renderer
    let mut screen_renderer = ScreenWindowRenderer::new(window.clone(), config, 0)?;

    // Create a test framebuffer
    let fb_config = FramebufferConfig {
        width: 400,
        height: 300,
        bpp: 32,
        stride: 400 * 4, // 4 bytes per pixel for RGBA32
        format: PixelFormat::RGBA32,
        software: true,
        scanline_pad: 32,
        little_endian: true,
    };

    let framebuffer = Framebuffer::new(fb_config)?;

    // Fill framebuffer with a test pattern
    println!("🎨 Creating test pattern in framebuffer");
    for y in 0..300 {
        for x in 0..400 {
            // Create a simple gradient pattern
            let r = (x * 255 / 400) as u8;
            let g = (y * 255 / 300) as u8;
            let b = ((x + y) * 255 / 700) as u8;
            let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            
            if let Err(e) = framebuffer.set_pixel(x, y, color) {
                eprintln!("Failed to set pixel ({}, {}): {}", x, y, e);
            }
        }
    }
    println!("✅ Test pattern created");

    let mut last_render_time = Instant::now();
    let render_interval = std::time::Duration::from_millis(16); // ~60 FPS
    let mut frame_count = 0u64;

    println!("🖼️  Starting event loop - you should see a gradient pattern");

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("🛑 Window close requested - shutting down");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                println!("📏 Window resized to {}x{}", size.width, size.height);
            }
            Event::AboutToWait => {
                // Render at regular intervals
                let now = Instant::now();
                if now.duration_since(last_render_time) >= render_interval {
                    window.request_redraw();
                    last_render_time = now;
                    frame_count += 1;

                    // Log frame rate occasionally
                    if frame_count % 300 == 0 {
                        println!("🖼️  Rendered {} frames", frame_count);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Render the framebuffer
                if let Err(e) = screen_renderer.render_framebuffer(&framebuffer) {
                    eprintln!("Failed to render framebuffer: {}", e);
                } else if frame_count % 120 == 0 {
                    println!("✅ Frame {} rendered successfully", frame_count);
                }
            }
            _ => {}
        }
    })
    .map_err(|e| rxserver::Error::Display(format!("Event loop error: {}", e)))?;

    println!("🖼️  Test completed");
    Ok(())
}
