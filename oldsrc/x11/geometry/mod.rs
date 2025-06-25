//! Geometric Calculations and Utilities
//!
//! This module provides geometric types and operations used throughout
//! the X11 server for window management, clipping, and transformations.

pub mod clipping;
pub mod intersection;
pub mod math;
pub mod regions;
pub mod transformations;
pub mod types;

// Re-export commonly used types and functions
pub use math::constants;
pub use regions::Region;
pub use types::*;
