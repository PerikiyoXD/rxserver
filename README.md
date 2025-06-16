# RX Server

Modern X11 server implementation in Rust with async architecture.

> **⚠️ Development Only - Core X11 protocol incomplete**

## Quick Start

```bash
git clone https://github.com/perikiyoxd/rxserver.git
cd rxserver
cargo build --release

# Run windowed mode
cargo run -- --mode windowed

# Test with X11 client
DISPLAY=:0 xterm
```

## Features

**Display Modes**
- **Windowed** - X11 desktop in a window on your existing desktop
- **Headless** - Protocol processing without visual output (testing/CI)
- **Hardware** - Direct display control (planned)

**Current Status**
- ✅ TCP connections and X11 handshake
- ✅ Extension queries (BIG-REQUESTS, XC-MISC)
- ✅ Basic window management and properties
- 🚧 Drawing operations (in progress)
- 🚧 Input handling (in progress)

**Architecture**
- Async I/O with Tokio
- Modular protocol handlers
- TOML configuration
- Structured logging

## Usage

```bash
# Basic usage
cargo run -- --mode windowed
cargo run -- --mode headless
cargo run -- --display :1

# With options
cargo run -- --width 1920 --height 1080 --verbose
cargo run -- --config custom.toml
```

**Command Line Options**
- `--mode` - windowed, headless, hardware (default: windowed)
- `--display` - Display number (default: :0)
- `--width/--height` - Screen dimensions (default: 1920x1080)
- `--config` - Configuration file (default: rxserver.toml)
- `--verbose` - Enable debug logging

## Configuration

Basic `rxserver.toml`:
```toml
[server]
max_clients = 10
display = ":0"

[display]
width = 1920
height = 1080

[logging]
level = "info"
```

## Development

**Project Structure**
```
src/
├── core/          # Config, logging, errors
├── protocol/      # X11 protocol handlers
├── server/        # Main server logic
├── network/       # Connection handling
├── window/        # Window management
└── graphics/      # Basic rendering
```

**Development Commands**
```bash
cargo check                # Quick validation
cargo test                 # Run tests
cargo clippy               # Lint code
RUST_LOG=debug cargo run   # Debug mode
```

**Protocol Handlers**
- Extension, Property, Window handlers for different request types
- Registry system coordinates handler routing
- Each handler implements `ProtocolHandler` trait

## Testing

```bash
# Unit tests
cargo test -- --nocapture

# Integration testing
cargo run -- --mode windowed --display :1 &
DISPLAY=:1 xclock
DISPLAY=:1 xterm
```

## Known Limitations

- Most X11 drawing operations missing
- No hardware acceleration
- Limited extension support
- Single-threaded rendering
- No security hardening

## Architecture Notes

**Current Implementation**
- Single windowed mode working reliably
- Protocol handler registry with fallback chains
- Basic resource management for windows/properties

**Design Decisions**
- Focus on core X11 protocol before advanced features
- Async-first design for client scalability
- Type-safe protocol handling with Rust's type system

## Contributing

1. Fork and create feature branch
2. Focus on core X11 protocol implementation
3. Add tests for new functionality
4. Follow existing code patterns
5. Update this README if adding major features

## Troubleshooting

**"Failed to bind to address"**
- Try different display: `--display :1`

**"Clients can't connect"**
- Check server is running: `ps aux | grep rxserver`
- Verify display: `echo $DISPLAY`

**Debug mode:**
```bash
RUST_LOG=trace cargo run -- --verbose
```

## Acknowledgements

- **The X.Org Foundation** for the comprehensive X Window System specification
- **The Rust Community** for exceptional tools, libraries, and ecosystem support
- **Tokio Contributors** for the outstanding async runtime


## License

MIT OR Apache-2.0