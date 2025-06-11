// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Timer module declarations and re-exports
//!
//! This module provides a clean interface to the timer functionality,
//! organizing the implementation across multiple files for better maintainability.

pub mod types;
pub mod timing;
pub mod constants;

// Re-export main types and constants for convenience
pub use types::Timer;
pub use constants::*;
