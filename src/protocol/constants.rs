use crate::protocol::WindowId;

pub const WINDOW_ID_ROOT: WindowId = 1;

/// The first major opcode value outside the core protocol's request range
/// (1-127). Every extension's major opcode is assigned from this point up
/// (see `protocol::extension_registry`).
pub const FIRST_EXTENSION_OPCODE: u8 = 128;
