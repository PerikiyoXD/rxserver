// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Timer type definitions

/// A high-precision timer for performance measurement
///
/// The Timer struct provides accurate timing capabilities using `std::time::Instant`
/// which offers monotonic time measurements suitable for performance profiling.
#[derive(Debug, Clone)]
pub struct Timer {
    /// The instant when the timer was started
    pub(crate) start: std::time::Instant,
    /// Optional instant when the timer was stopped
    pub(crate) stop: Option<std::time::Instant>,
    /// Human-readable name for this timer instance
    pub(crate) name: String,
}