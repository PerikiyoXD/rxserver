// -----------------------------------------------------------------------------
// Connection Setup request type
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSetupRequest {
    /// The byte order of the connection.
    /// 0x42 for MSB first, 0x6c for LSB first.
    pub byte_order: u8,
    /// The major version of the X11 protocol.
    pub protocol_major_version: u16,
    /// The minor version of the X11 protocol.
    pub protocol_minor_version: u16,
    /// The name of the authorization protocol.
    pub authorization_protocol_name: String,
    /// The data for the authorization protocol.
    pub authorization_protocol_data: Vec<u8>,
}

// -----------------------------------------------------------------------------
// InternAtom request type
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternAtomRequest {
    /// Indicates whether the atom should be created if it does not exist.
    pub only_if_exists: bool,
    /// The name of the atom to be interned.
    pub name: String,
}
