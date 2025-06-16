# RX Server - Modern Rust X11 Server Implementation

**RX Server** is a modern, plugin-based X11-compatible server implementation written in Rust, featuring multiple display modes, asynchronous architecture, and comprehensive protocol handling.

> [!WARNING]
> ### **DO NOT USE IN PRODUCTION UNDER ANY CIRCUMSTANCE!**
> 
> This project is in **active development** with foundational architecture complete but core X11 protocol implementation still in progress. Many essential X11 protocol features are missing or incomplete.
>
> ### **IGNORE THIS DISCLAIMER AT YOUR OWN RISK.**

## 🚀 Project Status

**Active Development** - Architecture and infrastructure are solid with multiple display modes operational. Core X11 protocol implementation is progressing with connection handshake, extension support, and basic window management working. Currently suitable for development and testing only.

## ✨ Features

### ✅ Implemented
- **Multi-Mode Display Support**
  - 🖥️ Virtual Display (windowed X11 server with native window rendering)
  - 🔲 Headless Mode (for testing/automation environments)
  - 🖼️ Native Display Mode (direct hardware access - infrastructure ready)
- **Core Infrastructure**
  - 🔧 Plugin-based architecture with dynamic registration and lifecycle management
  - ⚡ Full asynchronous I/O using Tokio for concurrent client handling
  - 📝 Comprehensive TOML-based configuration management with validation
  - 📊 Advanced structured logging with tracing and configurable output
  - 🔗 TCP connection handling with performance monitoring
  - 🎯 Command-line interface with extensive options
- **X11 Protocol Foundation**
  - 🤝 Complete connection handshake implementation with authentication
  - 🔌 Extension support framework (BIG-REQUESTS, XC-MISC operational)
  - 🏠 Window management infrastructure with hierarchy support
  - 📋 Property system for window attributes and atom management
  - 🎨 Graphics context framework and basic rendering pipeline
  - ⌨️ Input handling framework (keyboard/mouse event structure)
  - 📦 Protocol handler registry with specialized request routing
- **Resource Management**
  - 🪟 Window plugin with complete lifecycle management
  - 🔤 Font manager with caching and resource tracking
  - 🖱️ Cursor management system with handle allocation
  - ⚛️ Atom registry for X11 atom management and translation
  - 🔧 Performance monitoring and configuration tuning

### 🚧 In Progress
- **Core X11 Protocol Implementation**
  - Request parsing and validation for all major opcodes
  - Drawing operations (CreateWindow, MapWindow, ConfigureWindow)
  - Graphics operations (drawing primitives, text rendering)
  - Image and pixmap operations (CreatePixmap, PutImage, GetImage)
  - Event handling and client notification system
- **Display System Enhancement**
  - Virtual display rendering pipeline optimization
  - Hardware-accelerated graphics integration
  - Multi-screen and multi-monitor support
- **Advanced Window Management**
  - Complete window hierarchy and stacking
  - Focus management and input routing
  - Window decorations and compositing

### 📋 Planned
- **Extended X11 Protocol Support**
  - Complete X11 extension ecosystem (RENDER, XFIXES, COMPOSITE, etc.)
  - Advanced graphics operations and effects
  - Full input event handling with focus management
  - Keyboard and pointer grab operations
- **Performance and Optimization**
  - Hardware acceleration (OpenGL/Vulkan backends)
  - Memory management optimization and resource pooling
  - Multi-threaded rendering pipeline
  - Protocol compression and optimization
- **Production Features**
  - Comprehensive security hardening and access control
  - Unix domain socket support (Linux/macOS)
  - Extensive test suite with integration tests
  - Performance benchmarking and profiling tools
  - Documentation and usage examples

## 🏗️ Architecture

The RX server features a modular, plugin-based architecture with multiple display modes:

### Core Modules
- **`core/`** - Foundation (config, logging, error handling, CLI arguments)
- **`graphics/`** - Rendering contexts, basic graphics operations, and display management
- **`input/`** - Keyboard and mouse event handling and input routing
- **`network/`** - Connection management, TCP handling, and client session management
- **`plugins/`** - Plugin system with resource managers and lifecycle hooks
- **`protocol/`** - X11 protocol implementation with specialized request handlers and routing
- **`server/`** - Main server implementation with multi-mode display support and lifecycle management
- **`utils/`** - Development utilities, status tracking, and debugging tools
- **`window/`** - Window management, hierarchy, and property system

### Display Modes
- **Virtual Display** - Renders X11 output in a native window using winit/softbuffer (ideal for development/remote access)
- **Headless Mode** - No visual output, full protocol processing (perfect for testing/automation/CI-CD)
- **Native Display** - Direct hardware access with platform-specific backends (production-ready infrastructure)

### Protocol Handlers
- **Extension Handler** - Manages X11 extensions, queries, and feature negotiation
- **Property Handler** - Window properties, atoms, and attribute management
- **Window Handler** - Window operations, lifecycle, and hierarchy management
- **Virtual Display Handler** - Integrates with virtual display rendering system
- **Native Display Handler** - Hardware display management and direct rendering
- **Registry System** - Coordinates multiple protocol handlers and request routing

## 🛠️ Building

### Prerequisites

- **Rust 1.70+** (Developed and tested with Rust 1.87.0)
- **Platform Dependencies:**
  - Windows: No additional dependencies required
  - Linux: X11 development libraries (libx11-dev, libxext-dev) - optional for testing
  - macOS: Core Graphics framework access - automatic

### Build Commands

```bash
# Clone the repository
git clone https://github.com/perikiyoxd/rxserver.git
cd rxserver

# Build the project
cargo build

# Build optimized release version
cargo build --release

# Check for issues without building
cargo check

# Run tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Run linter with all checks
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

## ⚙️ Configuration

RX Server uses TOML configuration files. The default configuration file is `rxserver.toml`:

```bash
# Use the provided configuration template
cp rxserver.toml my-config.toml
# Edit as needed, then run:
cargo run -- --config my-config.toml
```

### Configuration Sections

- **`[server]`** - Server settings (max clients, display number, base port)
- **`[network]`** - Network configuration (listen address, IPv6 support, access control)
- **`[display]`** - Display settings (resolution, color depth, DPI, refresh rate)
- **`[graphics]`** - Graphics backend configuration (acceleration, rendering options)
- **`[logging]`** - Logging configuration (level, file output, structured format)
- **`[plugins]`** - Plugin system settings and plugin-specific configuration
- **`[security]`** - Access control, authentication, and security policies
- **`[performance]`** - Performance tuning (buffer sizes, connection limits, timeouts)

## 🚀 Running

### Basic Usage

```bash
# Run with default configuration (virtual display mode)
cargo run

# Run in different display modes
cargo run -- --mode virtual --width 1920 --height 1080
cargo run -- --mode headless
cargo run -- --mode native --width 1920 --height 1080

# Run on specific display
cargo run -- --display :1

# Use custom configuration
cargo run -- --config /path/to/config.toml

# Enable verbose logging
cargo run -- --verbose

# Run in foreground with debug output
cargo run -- --foreground --verbose
```

### Command Line Options

- `--display, -d` - Display number (default: :0)
- `--config, -c` - Configuration file path (default: rxserver.toml)
- `--mode, -m` - Display mode: virtual, headless, native (default: virtual)
- `--width` - Display width for virtual/native modes (default: 1920)
- `--height` - Display height for virtual/native modes (default: 1080)
- `--verbose, -v` - Enable verbose logging
- `--foreground, -f` - Run in foreground (don't daemonize)

### Display Modes Explained

- **Virtual Display**: Renders X11 output in a native window using winit and softbuffer - ideal for development, testing, and remote access scenarios
- **Headless**: No visual output, processes all X11 protocol requests - perfect for automated testing, CI/CD pipelines, and server environments
- **Native**: Direct hardware access with platform-specific display backends - for production deployments (infrastructure complete, implementation in progress)

## 🧪 Testing

```bash
# Run all tests with detailed output
cargo test -- --nocapture

# Run tests with tracing enabled
RUST_LOG=debug cargo test -- --nocapture

# Run specific test modules
cargo test protocol
cargo test handshake
cargo test window
cargo test extensions

# Run integration tests
cargo test --test integration

# Run benchmarks (if available)
cargo bench

# Generate code coverage report
cargo install cargo-llvm-cov
cargo llvm-cov --html
open target/llvm-cov/html/index.html
```

### Testing with Real X11 Clients

```bash
# Start the server in virtual display mode
cargo run -- --mode virtual --display :1

# In another terminal, test with X11 clients:
DISPLAY=:1 xterm
DISPLAY=:1 xclock
DISPLAY=:1 xeyes
```

## 📊 Development

### Project Structure

```
src/
├── main.rs                          # Application entry point
├── lib.rs                           # Library root with public API
├── core/                            # Core functionality
│   ├── args.rs                      # Command-line argument parsing
│   ├── config.rs                    # Configuration management
│   ├── error.rs                     # Error types and handling
│   └── logging.rs                   # Logging initialization
├── server/                          # Main server implementation
│   └── mod.rs                       # Server with multi-mode display support
├── protocol/                        # X11 protocol implementation
│   ├── types.rs                     # X11 data types and opcodes
│   ├── wire.rs                      # Wire protocol utilities
│   ├── handshake.rs                 # X11 connection handshake and authentication
│   ├── error.rs                     # Protocol-specific error types and handling
│   ├── traits.rs                    # Protocol handler traits and interfaces
│   └── handler/                     # Protocol request handlers
│       ├── mod.rs                   # Handler module exports
│       ├── extension.rs             # Extension queries (BIG-REQUESTS, XC-MISC)
│       ├── property.rs              # Property operations (GetProperty, ChangeProperty)
│       ├── window.rs                # Window operations and lifecycle management
│       ├── headless.rs              # Headless mode protocol handler
│       ├── virtual_display.rs       # Virtual display integration handler
│       ├── native_display.rs        # Native display hardware handler
│       ├── surface.rs               # Surface and drawing operations
│       ├── default.rs               # Default/fallback request handler
│       └── registry.rs              # Handler coordination and routing
├── network/                         # Network layer
│   ├── connection.rs                # Client connection management
│   └── mod.rs                       # Network module exports
├── plugins/                         # Plugin system
│   ├── registry.rs                  # Plugin registration, lifecycle, and management
│   ├── window.rs                    # Window management and lifecycle plugin
│   ├── font_manager.rs              # Font resource management and caching
│   ├── cursor_manager.rs            # Cursor management and handle allocation
│   ├── atom_registry.rs             # X11 atom management and translation
│   └── error.rs                     # Plugin-specific error types
├── window/                          # Window management system
│   ├── manager.rs                   # Window lifecycle, hierarchy, and stacking
│   ├── properties.rs                # Window properties, attributes, and metadata
│   └── mod.rs                       # Window module exports
├── graphics/                        # Graphics and rendering subsystem
│   ├── context.rs                   # Graphics context management and state
│   ├── renderer.rs                  # Basic rendering operations and primitives
│   ├── types.rs                     # Graphics types, primitives, and structures
│   └── mod.rs                       # Graphics module exports
├── input/                           # Input handling and event processing
│   ├── keyboard.rs                  # Keyboard event processing and mapping
│   ├── mouse.rs                     # Mouse event processing and tracking
│   └── mod.rs                       # Input module exports
└── utils/                           # Development utilities and helpers
    ├── implementation_status.rs     # Development status tracking and reporting
    ├── todo.rs                      # TODO macros and development utilities
    └── mod.rs                       # Utils module exports
```

### Development Workflow

1. **Check Implementation Status**
   ```bash
   # View current implementation status
   cargo run -- --verbose  # Shows status on startup
   ```

2. **Adding New Features**
   - Protocol handlers go in `src/protocol/handler/`
   - New plugins go in `src/plugins/`
   - Core functionality in respective modules
   - Update `implementation_status.rs` to track progress

3. **Testing Changes**
   ```bash
   cargo check           # Quick syntax/type checking
   cargo clippy          # Linting
   cargo test            # Run test suite
   cargo run -- --mode virtual  # Test with virtual display
   ```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes following the project patterns
4. Add tests for new functionality
5. Update implementation status in `utils/implementation_status.rs`
6. Ensure all tests pass (`cargo test`)
7. Format code (`cargo fmt`) and fix clippy warnings (`cargo clippy`)
8. Commit your changes (`git commit -m 'Add amazing feature'`)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

### Code Style Guidelines

- **Rust Standards**: Follow standard Rust formatting (`cargo fmt`)
- **Documentation**: Add doc comments for public APIs and complex functions
- **Error Handling**: Use the `ServerResult<T>` type for fallible operations
- **Logging**: Use `tracing` macros (`debug!()`, `info!()`, `warn!()`, `error!()`)
- **Testing**: Include unit tests for new functionality
- **TODOs**: Use the `todo_high!()`, `todo_medium!()`, `todo_low!()` macros for tracking

### Implementation Priorities

**High Priority** (blocking core functionality):
- Complete X11 request parsing and handling
- Window rendering and composition
- Core drawing operations

**Medium Priority** (important features):
- Font rendering and text operations
- Advanced window management
- Performance optimizations

**Low Priority** (nice to have):
- Advanced graphics features
- Additional X11 extensions
- Platform-specific optimizations

## 📋 Roadmap

### Phase 1: Core Foundation ✅
- [x] Basic server architecture and multi-mode display support
- [x] Plugin system with resource managers
- [x] X11 connection handshake implementation
- [x] Extension support (BIG-REQUESTS, XC-MISC)
- [x] Basic window management and properties
- [x] Configuration and logging systems

### Phase 2: Protocol Implementation 🚧
- [ ] Complete X11 core request/response handling
- [ ] Drawing operations (lines, rectangles, arcs, etc.)
- [ ] Pixmap creation and manipulation
- [ ] Graphics context operations
- [ ] Copy operations (CopyArea, CopyPlane)
- [ ] Text rendering and font operations

### Phase 3: Advanced Features 📋
- [ ] Image operations (PutImage, GetImage)
- [ ] Colormap management
- [ ] Advanced window operations
- [ ] Input event handling and focus management
- [ ] Grab operations (keyboard/pointer)

### Phase 4: Display Systems 🔄
- [ ] Complete virtual display rendering pipeline
- [ ] Native display hardware acceleration
- [ ] Multi-screen support
- [ ] OpenGL/Vulkan integration

### Phase 5: Production Ready ⏰
- [ ] Comprehensive error handling and recovery
- [ ] Security hardening and access control
- [ ] Performance optimization and profiling
- [ ] Extensive test suite and CI/CD
- [ ] Documentation and examples
- [ ] Packaging and distribution

## � Current Status

### Working Features
- ✅ TCP connection handling and client management
- ✅ X11 handshake and protocol negotiation
- ✅ Extension queries (QueryExtension, ListExtensions)
- ✅ Virtual display mode with native window rendering
- ✅ Basic window creation and management
- ✅ Property system (GetProperty, ChangeProperty)
- ✅ Plugin architecture with resource managers

### In Development
- 🔄 Core X11 drawing operations
- 🔄 Complete window management
- 🔄 Font and text rendering
- 🔄 Input event processing

### Known Limitations
- Most X11 drawing operations not implemented
- No hardware acceleration yet
- Limited extension support
- Unix socket support missing on Windows
- Performance not optimized for production use

## 🎯 Usage Examples

### Development and Testing

```bash
# Start development server with virtual display
cargo run -- --mode virtual --verbose

# Test with simple X11 clients
DISPLAY=:0 xterm &
DISPLAY=:0 xclock &
```

### Automated Testing

```bash
# Run headless for CI/CD pipelines
cargo run -- --mode headless --display :99

# Run automated tests against the server
DISPLAY=:99 your-test-suite
```

### Configuration Examples

```toml
# Example configuration for development
[display]
width = 1920
height = 1080
depth = 24

[server]
max_clients = 10

[logging]
level = "debug"
file = "rxserver.log"
```

## 📦 Dependencies

### Runtime Dependencies
- **tokio** - Async runtime and networking
- **tracing** - Structured logging and diagnostics
- **clap** - Command-line interface
- **serde/toml** - Configuration management
- **bytes** - Efficient byte handling for protocol parsing
- **winit/softbuffer** - Virtual display rendering

### Development Dependencies
- **tokio-test** - Async testing utilities

## 🔧 Troubleshooting

### Common Issues

**"Failed to bind to address"**
- Check if another X server is running on the same display
- Try a different display number: `--display :1`

**"Virtual display window doesn't appear"**
- Ensure you have a desktop environment running
- Check if the window is minimized or on another workspace

**"X11 clients can't connect"**
- Verify the server is running: `ps aux | grep rxserver`
- Check the correct display number: `echo $DISPLAY`
- Try with verbose logging: `--verbose`

### Debug Mode

```bash
# Run with maximum debugging information
RUST_LOG=trace cargo run -- --verbose --mode virtual

# Enable protocol-level debugging
RUST_LOG=rxserver::protocol=debug cargo run
```

## 📝 License

This project is licensed under the MIT OR Apache-2.0 license. See the [LICENSE](LICENSE) file for details.

## 🤝 Acknowledgments

- **The X.Org Foundation** for the comprehensive X Window System specification
- **The Rust Community** for exceptional tools, libraries, and ecosystem support
- **Tokio Contributors** for the outstanding async runtime
- **All Contributors** who help improve this project

## 📧 Contact

- **Project Repository**: [GitHub](https://github.com/yourusername/rxserver)
- **Issues & Bug Reports**: [GitHub Issues](https://github.com/yourusername/rxserver/issues)
- **Feature Requests**: [GitHub Discussions](https://github.com/yourusername/rxserver/discussions)

---

## 🔒 Security

This is a development project. If you discover security vulnerabilities, please report them responsibly through GitHub's security advisory system.

**Remember**: This project is not ready for production use. Use in development and testing environments only.