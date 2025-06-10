//! Window management
//!
//! This module handles window management operations including window hierarchy,
//! stacking order, and window properties.

pub mod manager;
pub mod properties;

pub use manager::*;
pub use properties::*;
