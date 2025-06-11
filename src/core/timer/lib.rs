// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Performance timing utility module
//!
//! This module provides high-precision timing capabilities for performance
//! measurement and debugging. The main component is the `Timer` struct which
//! can be used to measure elapsed time with nanosecond precision.
//!
//! # Examples
//!
//! ```rust
//! use crate::core::timer::Timer;
//!
//! // Basic timing
//! let timer = Timer::start("operation");
//! // ... perform some work ...
//! timer.stop(); // Logs the elapsed time
//!
//! // Get elapsed time as return value
//! let timer = Timer::start("calculation");
//! // ... perform calculation ...
//! let elapsed_ms = timer.stop_and_return();
//! ```

mod types;
mod timing;
mod constants;

// Public re-exports
pub use types::Timer;
pub use constants::*;