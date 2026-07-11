//! Assigns extension major opcodes once per server session, the way real
//! X servers do, instead of hardcoding a fixed number per extension.
//!
//! The spec does not mandate any particular major opcode for a given
//! extension - clients are required to learn it via `QueryExtension`. This
//! registry assigns sequential major opcodes (starting at 128, the first
//! value outside the core protocol's 1-127 range) to a fixed, ordered list
//! of extension names at construction time, so assignment is dynamic in the
//! sense that matters (nothing is hardcoded per-extension) while still being
//! reproducible run-to-run (same list, same order, every startup).

use std::collections::HashMap;

use crate::protocol::constants::FIRST_EXTENSION_OPCODE;

/// Extensions this server knows about, in assignment order. Not all of these
/// have a real implementation yet (see each handler for what's actually
/// backed) - they still get a major opcode assigned so `QueryExtension` can
/// answer consistently, and so real handlers can be added later without
/// changing opcode assignment for extensions ahead of them in this list.
const KNOWN_EXTENSIONS: &[&str] = &[
    "BIG-REQUESTS",
    "RANDR",
    "SHAPE",
    "MIT-SHM",
    "XINERAMA",
    "RENDER",
    "XKEYBOARD",
    "XInputExtension",
    "Generic Event Extension",
];

/// Extensions with an actual `RequestHandler` registered for their
/// sub-requests today. `QueryExtension` reports `present=0` for anything not
/// in this set, even though it still has a major opcode assigned above -
/// advertising "supported" for an extension nothing can actually handle
/// would just make every real request to it silently vanish.
const IMPLEMENTED_EXTENSIONS: &[&str] = &[
    "BIG-REQUESTS",
    "RANDR",
    "RENDER",
    "SHAPE",
    "XKEYBOARD",
    "XInputExtension",
    "Generic Event Extension",
];

#[derive(Debug, Clone)]
pub struct ExtensionRegistry {
    major_opcodes: HashMap<&'static str, u8>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        let mut major_opcodes = HashMap::new();
        let mut next_opcode: u8 = FIRST_EXTENSION_OPCODE;
        for name in KNOWN_EXTENSIONS {
            major_opcodes.insert(*name, next_opcode);
            next_opcode += 1;
        }
        Self { major_opcodes }
    }

    /// The major opcode assigned to `name` this session, if it's a known
    /// extension.
    pub fn major_opcode(&self, name: &str) -> Option<u8> {
        self.major_opcodes.get(name).copied()
    }

    /// Look up which known extension (if any) owns a given major opcode,
    /// for routing an incoming request byte to the right parser.
    pub fn extension_for_opcode(&self, major_opcode: u8) -> Option<&'static str> {
        self.major_opcodes
            .iter()
            .find(|&(_, &opcode)| opcode == major_opcode)
            .map(|(&name, _)| name)
    }

    /// Whether `name` has a real `RequestHandler` backing it, as opposed to
    /// just having a major opcode reserved.
    pub fn is_implemented(&self, name: &str) -> bool {
        IMPLEMENTED_EXTENSIONS.contains(&name)
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_every_known_extension_a_unique_opcode() {
        let registry = ExtensionRegistry::new();
        let mut opcodes: Vec<u8> = KNOWN_EXTENSIONS
            .iter()
            .map(|name| registry.major_opcode(name).unwrap())
            .collect();
        opcodes.sort_unstable();
        opcodes.dedup();
        assert_eq!(opcodes.len(), KNOWN_EXTENSIONS.len());
        assert!(opcodes.iter().all(|&op| op >= FIRST_EXTENSION_OPCODE));
    }

    #[test]
    fn unknown_extension_has_no_opcode() {
        let registry = ExtensionRegistry::new();
        assert_eq!(registry.major_opcode("NOT-A-REAL-EXTENSION"), None);
    }

    #[test]
    fn extension_for_opcode_round_trips() {
        let registry = ExtensionRegistry::new();
        for name in KNOWN_EXTENSIONS {
            let opcode = registry.major_opcode(name).unwrap();
            assert_eq!(registry.extension_for_opcode(opcode), Some(*name));
        }
    }

    #[test]
    fn only_big_requests_randr_render_shape_xkeyboard_xinput_and_xge_are_implemented() {
        let registry = ExtensionRegistry::new();
        assert!(registry.is_implemented("BIG-REQUESTS"));
        assert!(registry.is_implemented("RANDR"));
        assert!(registry.is_implemented("RENDER"));
        assert!(registry.is_implemented("SHAPE"));
        assert!(registry.is_implemented("XKEYBOARD"));
        assert!(registry.is_implemented("XInputExtension"));
        assert!(registry.is_implemented("Generic Event Extension"));
        assert!(!registry.is_implemented("MIT-SHM"));
        assert!(!registry.is_implemented("XINERAMA"));
    }
}
