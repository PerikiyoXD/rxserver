# RXServer Architecture

This document describes the organized architecture of the RXServer project.

## Project Structure

The project is organized into logical modules following Rust best practices:

```
src/
├── lib.rs              # Main library entry point
├── main.rs             # Binary entry point  
├── config.rs           # Configuration management
├── core/               # Core types and traits
│   ├── mod.rs
│   ├── errors.rs       # Error definitions
│   ├── ids.rs          # X11 ID management
│   └── traits.rs       # Common traits
├── protocol/           # X11 protocol implementation
│   ├── mod.rs
│   ├── types.rs        # X11 basic types
│   ├── opcodes.rs      # Request/response codes
│   ├── requests.rs     # Client requests
│   ├── responses.rs    # Server responses
│   ├── events.rs       # X11 events
│   ├── wire.rs         # Wire format serialization
│   └── response_builder.rs  # Response construction
├── server/             # Server implementation
│   ├── mod.rs
│   ├── core.rs         # Main server logic
│   ├── client.rs       # Client connection
│   ├── connection.rs   # Connection handling
│   ├── connection_manager.rs  # Connection management
│   ├── display.rs      # Display management
│   ├── display_manager.rs     # Display coordination
│   ├── event_loop.rs   # Main event loop
│   ├── events.rs       # Event handling
│   ├── handlers.rs     # Request handlers
│   ├── request_handler.rs     # Request processing
│   ├── resources.rs    # Resource management
│   └── state.rs        # Server state
├── window/             # Window management
│   ├── mod.rs
│   ├── manager.rs      # Window operations
│   └── properties.rs   # Window properties
├── graphics/           # Graphics and rendering
│   ├── mod.rs
│   ├── context.rs      # Graphics context
│   └── renderer.rs     # Rendering operations
├── input/              # Input handling
│   ├── mod.rs
│   ├── keyboard.rs     # Keyboard handling
│   └── mouse.rs        # Mouse handling
└── utils/              # Utility modules
    ├── mod.rs
    ├── logging.rs      # Logging utilities
    └── todo.rs         # Development helpers
```

## Design Principles

1. **Separation of Concerns**: Each module has a single responsibility
2. **Type Safety**: Use Rust's type system to prevent X11 protocol errors
3. **Async/Await**: Modern async handling for concurrent client connections
4. **Error Handling**: Comprehensive error handling with custom error types
5. **Modularity**: Clean module boundaries with minimal coupling

## Key Components

### Protocol Layer
- **Types**: Fundamental X11 data types
- **Events**: X11 events from server to client
- **Requests**: Client requests to server
- **Responses**: Server responses to client
- **Wire Format**: Binary serialization/deserialization

### Server Layer
- **Core**: Main server orchestration
- **Connections**: Client connection management
- **Event Loop**: Main server event processing
- **State**: Server state management

### Window Management
- **Manager**: Window creation, destruction, hierarchy
- **Properties**: Window attributes and properties

### Graphics
- **Context**: Graphics state and contexts
- **Renderer**: Drawing operations

### Input
- **Keyboard**: Key event processing
- **Mouse**: Pointer event processing

## Development Status

The codebase uses a todo system with priorities:
- `todo_critical!()` - Must fix immediately
- `todo_high!()` - Important for core functionality
- `todo_medium!()` - Nice to have features
- `todo_low!()` - Future enhancements

This helps track implementation progress and prioritize work.
