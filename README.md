# RX X11 Server

A modern, safe, and efficient implementation of the X11 Window System protocol in Rust, designed for cross-platform deployment with a focus on performance, modularity, and extensibility.

> **🚧 Early Development Phase** - Foundation complete, protocol implementation in progress

> **Note:** This project is not affiliated with the X.Org Foundation or any official X11 implementations.
> It is an independent effort to create a modern X11 server using Rust.

> **Looking for contributors!** If you are interested in X11, Rust, or systems programming, we would love your help in building this project. Check out the [Contributing](#contributing) section for details.


## Quick Start

```bash
git clone https://github.com/perikiyoxd/rxserver.git
cd rxserver
cargo build --release

# Run with virtual display (windowed X11 server)
cargo run -- --mode virtual --width 1920 --height 1080

# Run headless for testing/CI
cargo run -- --mode headless

# Test with X11 clients
DISPLAY=:0 xterm
DISPLAY=:0 xclock
```

## Architecture Overview

RX X11 Server follows a **layered, modular architecture** designed for maintainability, performance, and cross-platform compatibility:

### Core Domains

- **X11 Protocol Layer** (`src/x11/`) - Wire protocol, resource management, request processing
- **Display Backends** (`src/display/`) - Virtual, headless, and native rendering systems  
- **Platform Layer** (`src/platform/`) - OS-specific implementations (Windows, Linux, macOS)
- **Network Layer** (`src/network/`) - Connection management, transport protocols, discovery
- **Server Infrastructure** (`src/server/`) - Lifecycle, coordination, monitoring, plugin system

### Key Features

**🖥️ Multiple Display Modes**
- **Virtual Display** - X11 server running in a native window (development/testing)
- **Headless Mode** - Protocol processing without visual output (CI/testing)  
- **Native Display** - Direct hardware control (planned for production)

**🔧 Advanced Architecture**
- Async I/O with Tokio for high concurrency
- Plugin-based extensibility system
- Comprehensive resource management with XId allocation
- Multi-format configuration support (TOML, JSON, YAML, INI, XML)
- Advanced logging with rotation, filtering, and multiple outputs
- Health monitoring and diagnostics
- Security framework with authentication and authorization

**📡 Network Features**
- Multiple transport protocols (TCP, Unix sockets, named pipes, shared memory)
- Connection pooling and load balancing  
- Service discovery (mDNS, DNS-SD, broadcast)
- Proxy support (SSH tunneling, TCP proxy, load balancing)

## Current Implementation Status

### ✅ Completed Components

**Foundation & Infrastructure**
- Complete project architecture with modular design
- Async server framework with Tokio
- Comprehensive configuration system
- Advanced logging with multiple outputs and rotation
- Platform abstraction layer for Windows/Linux/macOS
- Plugin system for extensibility

**X11 Protocol Core**
- Wire protocol parsing and serialization
- Connection setup and handshake
- Request/response framework
- XId allocation and resource management
- Atom registry system
- Font management foundation
- Security and authentication framework

**Display Systems**
- Virtual display with winit + softbuffer
- Headless mode for testing
- Basic renderer framework
- Display backend abstraction

**Network Layer**
- Connection management and pooling
- Multiple transport protocols
- Service discovery mechanisms
- Monitoring and health checks

### 🚧 In Progress

**X11 Protocol Implementation**
- Core request handlers (CreateWindow, MapWindow, etc.)
- Event generation and delivery
- Property system implementation
- Graphics context management
- Drawing operations

**Input System**
- Device management and event processing
- Keyboard mapping (XKB integration planned)
- Mouse and touch input handling

### 📋 Planned Features

**Advanced X11 Features**
- Extension system (SHAPE, RENDER, DAMAGE, COMPOSITE, RANDR)
- Advanced graphics operations
- Multi-screen support
- Window composition and damage tracking

**Performance & Production**
- Hardware-accelerated rendering
- Memory optimization
- Performance profiling and benchmarks
- Security hardening

## Usage

### Command Line Interface

```bash
# Virtual display mode (development/testing)
cargo run -- --mode virtual --width 1920 --height 1080 --display :1

# Headless mode (CI/testing)
cargo run -- --mode headless --display :2

# Native display mode (future)
cargo run -- --mode native --width 1920 --height 1080

# Custom configuration
cargo run -- --config custom.toml --verbose
```

**Command Line Options**
- `--mode` - Display mode: `virtual`, `headless`, `native` (default: virtual)
- `--display` - Display number (default: :0)  
- `--width/--height` - Display dimensions (default: 1920x1080)
- `--config` - Configuration file path (default: rxserver.toml)
- `--verbose` - Enable verbose logging

### Configuration

**Example `rxserver.toml`:**
```toml
[server]
max_clients = 50
display = ":1"
plugin_directory = "./plugins"

[display]
backend = "virtual"  # virtual, headless, native
width = 1920
height = 1080
depth = 24

[network]
bind_address = "127.0.0.1:6001"
enable_unix_socket = true
connection_timeout = 30

[logging]
level = "info"
output = "both"  # console, file, both
rotation = "daily"
max_size = "100MB"

[security]
authentication_required = false
allowed_hosts = ["localhost", "127.0.0.1"]
```

## Development

### Project Structure

```
src/
├── x11/                 # X11 protocol implementation
│   ├── protocol/        # Wire format, parsing, serialization
│   ├── resources/       # XId allocation, resource management
│   ├── requests/        # Request handlers and validation
│   ├── events/          # Event generation and delivery
│   ├── security/        # Authentication and authorization
│   └── state/           # Server state management
├── display/             # Display backend abstraction
│   └── backend/         # Virtual, headless, native backends
├── server/              # Server infrastructure
│   ├── lifecycle/       # Startup, shutdown, restart
│   ├── coordination/    # Service orchestration
│   ├── monitoring/      # Health checks, telemetry
│   └── plugins/         # Plugin system
├── network/             # Network layer
│   ├── connection/      # Connection management
│   ├── transport/       # Protocol implementations
│   └── discovery/       # Service discovery
├── platform/            # OS-specific implementations
├── config/              # Configuration management  
├── logging/             # Advanced logging system
├── fonts/               # Font management
├── input/               # Input device handling
└── diagnostics/         # Health monitoring and debugging
```

### Development Commands

```bash
# Development workflow
cargo check                    # Fast syntax checking
cargo test                     # Run test suite
cargo test -- --nocapture     # Tests with output
cargo clippy                   # Linting and suggestions
cargo fmt                      # Code formatting

# Running with debug output
RUST_LOG=debug cargo run -- --mode virtual --verbose
RUST_LOG=trace cargo run -- --mode headless --verbose

# Performance profiling
cargo build --release --features profiling
cargo run --release -- --mode virtual --width 3840 --height 2160
```

### Testing Strategy

**Unit Tests**
```bash
cargo test x11::protocol::parser  # Protocol parsing tests
cargo test server::lifecycle      # Server lifecycle tests  
cargo test network::connection    # Connection management tests
```

**Integration Tests**
```bash
# Start server in background
cargo run -- --mode virtual --display :99 &

# Test with X11 applications
DISPLAY=:99 xterm &
DISPLAY=:99 xclock &
DISPLAY=:99 xeyes &
```

**Performance Testing**
```bash
cargo run --release -- --mode headless &
# Run X11 benchmark suites
```

## Protocol Implementation Details

### Supported X11 Requests

**Connection Management**
- ✅ Connection setup and handshake
- ✅ Authentication and authorization
- ✅ Extension querying

**Core Requests** (In Progress)
- 🚧 CreateWindow, DestroyWindow
- 🚧 MapWindow, UnmapWindow  
- 🚧 ConfigureWindow
- 🚧 CreateGC, ChangeGC, FreeGC
- 🚧 InternAtom, GetAtomName
- 🚧 OpenFont, CloseFont

**Resource Management**
- ✅ XId allocation and tracking
- ✅ Resource lifecycle management
- ✅ Client cleanup on disconnect

### Display Backend Comparison

| Feature | Virtual | Headless | Native |
|---------|---------|----------|--------|
| Visual Output | ✅ Window | ❌ None | 🚧 Direct HW |
| Performance | Medium | High | Highest |
| Development | ✅ Ideal | ✅ CI/Testing | 🚧 Production |
| Cross-Platform | ✅ Yes | ✅ Yes | 🚧 Platform-specific |

## Performance Characteristics

- **Async I/O**: Handle thousands of concurrent connections
- **Zero-copy Parsing**: Efficient wire protocol processing  
- **Resource Pooling**: Minimize allocation overhead
- **Lazy Loading**: Load components on demand
- **Damage Tracking**: Minimal rendering updates (planned)

## Contributing

We welcome contributions! Please see our contribution guidelines:

1. **Fork and Branch**: Create feature branches from `nightly`
2. **Architecture**: Follow the established modular design
3. **Testing**: Add comprehensive tests for new functionality
4. **Documentation**: Update relevant documentation
5. **Code Quality**: Run `cargo clippy` and `cargo fmt`

### Priority Areas

- X11 protocol request handlers
- Event system implementation  
- Graphics and rendering optimizations
- Platform-specific enhancements
- Performance optimizations
- Test coverage improvements

## Troubleshooting

**Connection Issues**
```bash
# Check if server is running
ps aux | grep rxserver

# Try different display number
cargo run -- --display :99

# Check binding conflicts
netstat -ln | grep 6000
```

**Performance Issues**
```bash
# Enable performance logging
RUST_LOG=rxserver::server::monitoring=debug cargo run

# Run with release optimizations
cargo run --release
```

**Debug Mode**
```bash
# Maximum verbosity
RUST_LOG=trace cargo run -- --verbose

# Specific module debugging
RUST_LOG=rxserver::x11::protocol=debug cargo run
```

## Roadmap

### Phase 1: Foundation (✅ Complete)
- Core architecture and infrastructure
- Basic protocol framework
- Display backend abstraction

### Phase 2: Core Protocol (🚧 Current)
- Essential X11 request handling
- Resource management 
- Basic window operations

### Phase 3: Display & Input (📋 Next)
- Complete rendering pipeline
- Input device integration
- Event system completion

### Phase 4: Extensions & Production (📋 Future)
- X11 extensions support
- Performance optimization
- Production deployment features

## License

This project is dual-licensed under:
- **MIT License** - See [LICENSE-MIT](LICENSE-MIT)
- **Apache License 2.0** - See [LICENSE-APACHE](LICENSE-APACHE)

## Acknowledgments

- **The X.Org Foundation** - X11 protocol specification and reference implementations
- **Rust Community** - Outstanding ecosystem and development tools
- **Tokio Project** - High-performance async runtime
- **winit** and **softbuffer** - Cross-platform windowing and rendering