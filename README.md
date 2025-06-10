# RX - Rust X Window System

**RX** is a modern, safe, and efficient implementation of the X11 protocol written in Rust. This project aims to provide a full-featured X Window System server with improved security, performance, and maintainability.

## 🚀 Project Status

**Early Development** - This project is in the initial development phase. Core architecture is established, but many features are still being implemented.

## ✨ Features

### Implemented
- ✅ Modular architecture with clear separation of concerns
- ✅ Asynchronous I/O using Tokio for handling multiple client connections
- ✅ Basic X11 protocol structure (requests, responses, events)
- ✅ Window management framework
- ✅ Graphics context management
- ✅ Input handling framework (keyboard and mouse)
- ✅ Configuration management
- ✅ Comprehensive logging system

### Planned
- 🚧 Complete X11 protocol implementation
- 🚧 Network connection handling (TCP and Unix sockets)
- 🚧 Window rendering and composition
- 🚧 Font handling and text rendering
- 🚧 Graphics acceleration support
- 🚧 X11 extensions support
- 🚧 Performance optimizations

## 🏗️ Architecture

The RX server is organized into several key modules:

- **`protocol/`** - X11 protocol implementation (requests, responses, events, types)
- **`server/`** - Core server implementation and client connection management
- **`window/`** - Window management and properties
- **`graphics/`** - Graphics rendering and context management
- **`input/`** - Input handling (keyboard and mouse events)
- **`config/`** - Configuration management
- **`utils/`** - Utility modules and helper functions

## 🛠️ Building

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Build Commands

```bash
# Clone the repository
git clone <repository-url>
cd rxserver

# Build the project
cargo build

# Build in release mode
cargo build --release

# Run the server (development)
cargo run

# Run with custom configuration
cargo run -- --config config.example.toml --display :1 --verbose
```

## ⚙️ Configuration

RX uses TOML configuration files. Copy `config.example.toml` to `config.toml` and modify as needed:

```bash
cp config.example.toml config.toml
```

### Configuration Sections

- **`[server]`** - Server settings (max clients, ports, sockets)
- **`[display]`** - Display settings (resolution, color depth, DPI)
- **`[graphics]`** - Graphics backend settings
- **`[input]`** - Input device settings
- **`[logging]`** - Logging configuration

## 🚀 Running

### Basic Usage

```bash
# Start on display :0 (default)
./target/release/rxserver

# Start on specific display
./target/release/rxserver --display :1

# Run in foreground with verbose logging
./target/release/rxserver --foreground --verbose

# Use custom configuration
./target/release/rxserver --config /path/to/config.toml
```

### Command Line Options

- `--display, -d` - Display number (default: :0)
- `--config, -c` - Configuration file path (default: config.toml)
- `--verbose, -v` - Enable verbose logging
- `--foreground, -f` - Run in foreground (don't daemonize)

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test protocol::tests

# Run with coverage (requires cargo-llvm-cov)
cargo llvm-cov
```

## 📊 Development

### Code Organization

```
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library root with public API
├── config.rs               # Configuration management
├── protocol/               # X11 protocol implementation
│   ├── mod.rs              # Protocol module root
│   ├── types.rs            # X11 data types and constants
│   ├── requests.rs         # Request parsing and handling
│   ├── responses.rs        # Response generation
│   └── events.rs           # Event handling
├── server/                 # Core server implementation
│   ├── mod.rs              # Server module root
│   ├── connection.rs       # Client connection management
│   ├── display.rs          # Display/screen management
│   └── resources.rs        # Resource management
├── window/                 # Window management
│   ├── mod.rs              # Window module root
│   ├── manager.rs          # Window manager logic
│   └── properties.rs       # Window properties
├── graphics/               # Graphics and rendering
│   ├── mod.rs              # Graphics module root
│   ├── context.rs          # Graphics context management
│   └── renderer.rs         # Basic rendering operations
├── input/                  # Input handling
│   ├── mod.rs              # Input module root
│   ├── keyboard.rs         # Keyboard event handling
│   └── mouse.rs            # Mouse event handling
└── utils/                  # Utilities
    ├── mod.rs              # Utils module root
    └── logging.rs          # Logging configuration
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Run Clippy for linting (`cargo clippy`)
- Add documentation for public APIs
- Include unit tests for new functionality

## 📋 Roadmap

### Phase 1: Core Protocol (Current)
- [ ] Complete X11 connection setup
- [ ] Basic request/response handling
- [ ] Window creation and management
- [ ] Basic graphics operations

### Phase 2: Essential Features
- [ ] Complete protocol implementation
- [ ] Font handling
- [ ] Pixmap operations
- [ ] Copy operations

### Phase 3: Advanced Features
- [ ] X11 extensions support
- [ ] Hardware acceleration
- [ ] Performance optimizations
- [ ] Security enhancements

### Phase 4: Production Ready
- [ ] Comprehensive testing
- [ ] Documentation
- [ ] Packaging
- [ ] Distribution

## 🐛 Known Issues

- Unix socket support not implemented on Windows
- Hardware acceleration not yet available
- Limited X11 extension support
- Performance not yet optimized

## 📝 License

This project is licensed under the MIT OR Apache-2.0 license. See the [LICENSE](LICENSE) file for details.

## 🤝 Acknowledgments

- The X.Org Foundation for the X Window System specification
- The Rust community for excellent tools and libraries
- Contributors to the Tokio async runtime

## 📧 Contact

- Project Repository: [GitHub](https://github.com/yourusername/rxserver)
- Issues: [GitHub Issues](https://github.com/yourusername/rxserver/issues)

---

**Note**: This is an experimental project. Use at your own risk in production environments.