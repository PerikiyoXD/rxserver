//! Core types and traits for RXServer
//!
//! This module contains fundamental types, traits, and patterns used throughout
//! the X server implementation. It establishes the base architecture and
//! provides type-safe abstractions over X11 concepts.

pub mod ids;
pub mod traits;
pub mod errors;

pub use ids::*;
pub use traits::*;
pub use errors::*;
