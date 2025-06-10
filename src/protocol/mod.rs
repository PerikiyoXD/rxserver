//! X11 protocol implementation
//!
//! This module contains the core X11 protocol handling, including request parsing,
//! response generation, event handling, and data type definitions.

pub mod events;
pub mod requests;
pub mod responses;
pub mod types;

pub use events::*;
pub use requests::*;
pub use responses::*;
pub use types::*;

use crate::{Error, Result};
