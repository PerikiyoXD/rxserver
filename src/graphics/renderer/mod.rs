// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Renderer module declarations and re-exports
//!
//! This module provides a clean interface to the renderer functionality,
//! organizing the implementation across multiple files for better maintainability.

pub mod types;
pub mod rendering;
pub mod utils;

// Re-export main types for convenience
pub use types::Renderer;
