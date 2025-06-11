// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Timing constants and conversion factors

/// Milliseconds per second conversion factor
/// 
/// Used to convert Duration values to milliseconds for human-readable output
pub const MS_PER_SECOND: f64 = 1_000.0;

/// Microseconds per second conversion factor
/// 
/// Used for high-precision timing measurements when microsecond accuracy is needed
pub const US_PER_SECOND: f64 = 1_000_000.0;

/// Nanoseconds per second conversion factor
/// 
/// Used for the highest precision timing measurements available
pub const NS_PER_SECOND: f64 = 1_000_000_000.0;
