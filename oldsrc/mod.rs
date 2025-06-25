//! RXServer - A CLEAN Architecture X11 Server Implementation
//!
//! This X11 server follows CLEAN architecture principles with strict separation of concerns,
//! dependency inversion, and modular design for cross-platform compatibility.

pub mod types;

// Core X11 protocol domain
pub mod x11;

// Display abstraction layer
pub mod display;

// Platform abstraction layer
pub mod platform;

// Network transport layer
pub mod network;

// Font management system
pub mod fonts;

// Input handling system
pub mod input;

// Configuration management
pub mod config;

// Security framework
pub mod security;

// Performance monitoring and optimization
pub mod performance;

// Testing infrastructure
pub mod testing;

// Logging system
pub mod logging;

// Diagnostics and health monitoring
pub mod diagnostics;

// Server coordination and lifecycle
pub mod server;

// Plugin and extension system
pub mod plugins;

// Runtime execution environment
pub mod runtime;

// Service discovery and management
pub mod services;

// Monitoring and telemetry
pub mod monitoring;

// Server API interfaces
pub mod api;

// Server lifecycle management
pub mod lifecycle;

// Component coordination
pub mod coordination;

pub use types::*;
