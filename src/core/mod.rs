//! Core types and traits for RXServer
//!
//! This module contains fundamental types, traits, and patterns used throughout
//! the X server implementation. It establishes the base architecture and
//! provides type-safe abstractions over X11 concepts.

pub mod atom_manager;
pub mod cursor_manager;
pub mod errors;
pub mod font_manager;
pub mod ids;
pub mod pointer_manager;
pub mod traits;

pub use atom_manager::AtomManager;
pub use cursor_manager::CursorManager;
pub use errors::*;
pub use font_manager::FontManager;
pub use ids::*;
pub use pointer_manager::PointerManager;
pub use traits::*;
