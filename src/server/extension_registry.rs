//! X11 Extension Registry
//!
//! This module manages X11 extensions available in the rxserver.
//! It provides a central registry for extensions and their opcodes.

use std::collections::HashMap;
use tracing::debug;

/// Information about an X11 extension
#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    /// Extension name (e.g. "BIG-REQUESTS", "MIT-SHM")
    pub name: String,
    /// Major opcode assigned to this extension
    pub major_opcode: u8,
    /// First event number for this extension (0 if no events)
    pub first_event: u8,
    /// First error number for this extension (0 if no errors)
    pub first_error: u8,
}

/// X11 Extension Registry
/// 
/// Manages all extensions available in the rxserver.
/// Based on the standard X11 extension system.
pub struct ExtensionRegistry {
    /// Extensions by name
    extensions: HashMap<String, ExtensionInfo>,
    /// Next available major opcode
    next_major_opcode: u8,
    /// Next available event number
    next_event: u8,
    /// Next available error number
    next_error: u8,
}

impl ExtensionRegistry {
    /// Create a new extension registry with standard X11 extensions
    pub fn new() -> Self {
        let mut registry = Self {
            extensions: HashMap::new(),
            next_major_opcode: 128, // Extension opcodes start at 128
            next_event: 64,         // Extension events start at 64
            next_error: 128,        // Extension errors start at 128
        };

        // Register common X11 extensions that rxserver supports
        registry.register_standard_extensions();
        
        registry
    }

    /// Register standard X11 extensions
    fn register_standard_extensions(&mut self) {
        // Core extensions that most X11 clients expect
        self.register_extension("BIG-REQUESTS", 0, 0);
        self.register_extension("XC-MISC", 0, 0);
        self.register_extension("MIT-SHM", 1, 1); // Has events and errors
        self.register_extension("SHAPE", 1, 1);
        self.register_extension("SYNC", 2, 3);
        self.register_extension("XFIXES", 1, 1);
        self.register_extension("RENDER", 0, 1);
        self.register_extension("RANDR", 2, 4);
        self.register_extension("Generic Event Extension", 0, 0);
        self.register_extension("Present", 4, 1);
        self.register_extension("DRI2", 1, 5);
        self.register_extension("DRI3", 0, 2);
        self.register_extension("GLX", 0, 13); // GLX has many error codes
        self.register_extension("XInputExtension", 15, 6); // XI has many events and errors
        self.register_extension("XKEYBOARD", 1, 0);
        self.register_extension("XTEST", 0, 1);
        self.register_extension("RECORD", 0, 5);
        self.register_extension("SECURITY", 1, 2);
        self.register_extension("DPMS", 0, 0);
        self.register_extension("DOUBLE-BUFFER", 1, 1);
        self.register_extension("Composite", 0, 1);
        self.register_extension("DAMAGE", 1, 1);
        self.register_extension("X-Resource", 0, 0);
        self.register_extension("SELinux", 0, 3);
    }

    /// Register a new extension
    fn register_extension(&mut self, name: &str, num_events: u8, num_errors: u8) {
        let major_opcode = self.next_major_opcode;
        let first_event = if num_events > 0 { 
            let event = self.next_event;
            self.next_event += num_events;
            event
        } else { 
            0 
        };
        let first_error = if num_errors > 0 { 
            let error = self.next_error;
            self.next_error += num_errors;
            error
        } else { 
            0 
        };

        let info = ExtensionInfo {
            name: name.to_string(),
            major_opcode,
            first_event,
            first_error,
        };

        debug!(
            "Registered extension '{}': major_opcode={}, first_event={}, first_error={}",
            name, major_opcode, first_event, first_error
        );

        self.extensions.insert(name.to_string(), info);
        self.next_major_opcode += 1;
    }

    /// Query an extension by name
    pub fn query_extension(&self, name: &str) -> Option<&ExtensionInfo> {
        self.extensions.get(name)
    }

    /// Get all registered extensions
    pub fn list_extensions(&self) -> Vec<&ExtensionInfo> {
        self.extensions.values().collect()
    }

    /// Check if an extension is registered
    pub fn has_extension(&self, name: &str) -> bool {
        self.extensions.contains_key(name)
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
    fn test_extension_registry() {
        let registry = ExtensionRegistry::new();
        
        // Test querying existing extension
        let shm = registry.query_extension("MIT-SHM").unwrap();
        assert_eq!(shm.name, "MIT-SHM");
        assert!(shm.major_opcode >= 128);
        assert!(shm.first_event > 0);
        assert!(shm.first_error > 0);

        // Test querying non-existent extension
        assert!(registry.query_extension("NON-EXISTENT").is_none());

        // Test listing extensions
        let extensions = registry.list_extensions();
        assert!(!extensions.is_empty());
        assert!(extensions.iter().any(|e| e.name == "MIT-SHM"));
        assert!(extensions.iter().any(|e| e.name == "BIG-REQUESTS"));
    }

    #[test]
    fn test_has_extension() {
        let registry = ExtensionRegistry::new();
        
        assert!(registry.has_extension("MIT-SHM"));
        assert!(registry.has_extension("BIG-REQUESTS"));
        assert!(!registry.has_extension("NON-EXISTENT"));
    }
}
