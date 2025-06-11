# RXServer Centralized Logging System

This document explains how to use the newly implemented centralized logging system in RXServer.

## Overview

The centralized logging system provides a unified approach to logging across the entire RXServer codebase. It's organized into focused modules and provides both structured logging macros and component-specific utilities.

## Module Structure

```
src/logging/
├── mod.rs              # Main module exports
├── init.rs             # Logging initialization and setup
├── macros.rs           # Structured logging macros
├── components.rs       # Component-specific logging utilities
├── utils.rs            # Common logging patterns and helpers
├── types.rs            # LogLevel enum definitions
├── lib.rs              # Legacy compatibility functions
└── logging.rs          # Statistics and monitoring utilities
```

## Quick Start

### 1. Initialize Logging

In your main function or early in your application:

```rust
use rxserver::logging::init_logging;
use rxserver::config::types::LoggingSettings;

// Initialize with configuration
let config = ServerConfig::load("config.toml")?;
init_logging(&config.logging)?;
```

### 2. Basic Component Logging

Use the component-specific loggers for standardized output:

```rust
use rxserver::logging::{ServerLogger, NetworkLogger, ProtocolLogger};

// Server events
ServerLogger::startup(0, "config.toml");
ServerLogger::config_loaded("/path/to/config.toml");
ServerLogger::bind_address("127.0.0.1:6000");
ServerLogger::shutdown();

// Network events
NetworkLogger::connection_accepted(client_id, remote_addr);
NetworkLogger::connection_closed(client_id, "client disconnected");
NetworkLogger::listener_started("127.0.0.1:6000");

// Protocol events
ProtocolLogger::request_received(client_id, "CreateWindow");
ProtocolLogger::response_sent(client_id, "CreateWindowResponse");
ProtocolLogger::handshake_completed(client_id, 11, 0);
```

### 3. Structured Logging Macros

Use macros for consistent structured logging:

```rust
// Connection events
log_connection!(accept, client_id, socket_addr);
log_connection!(disconnect, client_id, "timeout");
log_connection!(error, client_id, &error);

// X11 Protocol events
log_x11_protocol!(request, client_id, "CreateWindow");
log_x11_protocol!(response, client_id, "CreateWindowResponse");
log_x11_protocol!(error, client_id, 1, "BadWindow");

// Resource management
log_resource!(create, "window", window_id, client_id);
log_resource!(destroy, "pixmap", pixmap_id, client_id);
log_resource!(leak, "font", font_id, client_id);

// Performance metrics
log_performance!(timing, "request_processing", 1.5);
log_performance!(memory, "client_buffers", 1024);
log_performance!(counter, "requests_per_second", 150.0);

// Implementation status
log_implementation!(complete, "tcp_listener", "TCP connections working");
log_implementation!(partial, "window_manager", "Basic framework implemented");
log_implementation!(todo, "unix_sockets", "Need to implement Unix domain sockets");
log_implementation!(missing, "hardware_accel", "No GPU acceleration support");
```

### 4. Utility Functions

Use utility functions for common patterns:

```rust
use rxserver::logging::utils::{with_context, with_timing, log_statistics};

// Execute with context logging
with_context("initializing server", || {
    // Your initialization code here
});

// Execute with timing
let result = with_timing("process_request", || {
    // Your code to time
    process_x11_request()
});

// Log statistics
let stats = [
    ("total_connections", 42),
    ("active_connections", 3),
    ("requests_processed", 1337),
];
log_statistics("Server", &stats);
```

### 5. Direct Tracing Usage

For custom logging needs, use tracing directly:

```rust
use tracing::{info, warn, error, debug, trace};

info!(client_id = 123, request = "CreateWindow", "Processing client request");
warn!(error = %err, "Non-fatal error occurred");
error!(component = "display", "Critical display initialization failed");
debug!(bytes = buffer.len(), "Buffer processed");
trace!("Detailed execution trace");
```

## Configuration

The logging system uses the existing `LoggingSettings` configuration:

```toml
[logging]
level = "info"           # error, warn, info, debug, trace
stdout = true            # Log to stdout/stderr
file = "server.log"      # Optional log file path
max_file_size = 100      # MB before rotation
rotation_count = 5       # Number of rotated files to keep
```

## Features

### ✅ Implemented Features

- **Unified initialization** using existing configuration
- **Component-specific loggers** for consistent formatting
- **Structured logging macros** for different event types
- **Performance timing utilities**
- **Statistics logging helpers**
- **Error context propagation**
- **Rate-limited warnings** (prevents log spam)
- **Legacy log crate compatibility** via LogTracer bridge

### 🚧 Available for Extension

- **File logging** (configuration exists, implementation needed)
- **Log rotation** (configuration exists, implementation needed)
- **Custom formatters** for different output formats
- **Metrics collection** integration
- **Distributed tracing** support

## Examples in Production

### Server Startup Sequence
```rust
// 1. Initialize logging first
init_logging(&config.logging)?;

// 2. Log server startup
ServerLogger::startup(config.server.display_number, &config_path);

// 3. Log system information
log_system_info();

// 4. Log implementation status
log_implementation_status();

// 5. Start components with logging
NetworkLogger::listener_started(&bind_address);
```

### Client Connection Handling
```rust
// Accept connection
let stream = listener.accept().await?;
let client_id = assign_client_id();
NetworkLogger::connection_accepted(client_id, stream.peer_addr()?);

// Process handshake
with_timing("x11_handshake", || {
    let (major, minor) = perform_handshake(&stream)?;
    ProtocolLogger::handshake_completed(client_id, major, minor);
    Ok(())
})?;

// Handle request
loop {
    let request = parse_request(&stream)?;
    ProtocolLogger::request_received(client_id, &request.name());
    
    let response = with_timing("process_request", || {
        handle_request(client_id, request)
    })?;
    
    ProtocolLogger::response_sent(client_id, &response.name());
}
```

### Error Handling with Context
```rust
fn process_create_window(client_id: u32, request: CreateWindowRequest) -> Result<()> {
    // Log resource creation
    log_resource!(create, "window", request.window_id, client_id);
    
    // Process with error context
    create_window_internal(request)
        .map_err(|e| log_and_return_error(e, "create_window_processing"))?;
    
    ProtocolLogger::response_sent(client_id, "CreateWindowResponse");
    Ok(())
}
```

## Benefits

1. **Consistency** - All components use the same logging patterns
2. **Structured Data** - Rich context in logs for better debugging
3. **Performance** - Built-in timing and performance metrics
4. **Maintainability** - Centralized configuration and utilities
5. **Extensibility** - Easy to add new component loggers and macros
6. **Compatibility** - Works with existing log crate usage via bridge

The logging system is now ready for production use and can be extended as needed for specific RXServer requirements.
