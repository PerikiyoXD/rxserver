//! Message framing implementation
//!
//! Handles message framing and deframing for network protocols.

use crate::network::ConnectionId;
use std::collections::VecDeque;
use tracing::{debug, error, warn};

/// Frame type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum FrameType {
    /// X11 request frame
    Request,
    /// X11 response frame
    Response,
    /// X11 event frame
    Event,
    /// X11 error frame
    Error,
    /// Control frame (for protocol management)
    Control,
    /// Data continuation frame
    Continuation,
}

/// Frame structure
#[derive(Debug, Clone)]
pub struct Frame {
    /// Frame type
    pub frame_type: FrameType,
    /// Frame sequence number
    pub sequence: u16,
    /// Frame payload
    pub payload: Vec<u8>,
    /// Frame flags
    pub flags: FrameFlags,
}

/// Frame flags
#[derive(Debug, Clone)]
pub struct FrameFlags {
    /// Frame is fragmented
    pub fragmented: bool,
    /// This is the last fragment
    pub last_fragment: bool,
    /// Frame requires response
    pub requires_response: bool,
    /// Frame is compressed
    pub compressed: bool,
    /// Frame is encrypted
    pub encrypted: bool,
}

impl Default for FrameFlags {
    fn default() -> Self {
        Self {
            fragmented: false,
            last_fragment: true,
            requires_response: false,
            compressed: false,
            encrypted: false,
        }
    }
}

/// Framing error
#[derive(Debug, thiserror::Error)]
pub enum FramingError {
    #[error("Invalid frame header: {0}")]
    InvalidHeader(String),

    #[error("Frame too large: {0} bytes (max: {1})")]
    FrameTooLarge(usize, usize),

    #[error("Incomplete frame: need {0} more bytes")]
    IncompleteFrame(usize),

    #[error("Invalid frame type: {0}")]
    InvalidFrameType(u8),

    #[error("Frame sequence error: expected {0}, got {1}")]
    SequenceError(u16, u16),

    #[error("Buffer overflow")]
    BufferOverflow,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Frame processor configuration
#[derive(Debug, Clone)]
pub struct FrameProcessorConfig {
    /// Maximum frame size in bytes
    pub max_frame_size: usize,
    /// Buffer size for incoming data
    pub buffer_size: usize,
    /// Enable frame compression
    pub enable_compression: bool,
    /// Enable frame encryption
    pub enable_encryption: bool,
    /// Maximum number of queued frames
    pub max_queued_frames: usize,
}

impl Default for FrameProcessorConfig {
    fn default() -> Self {
        Self {
            max_frame_size: 1024 * 1024, // 1MB
            buffer_size: 64 * 1024,      // 64KB
            enable_compression: false,
            enable_encryption: false,
            max_queued_frames: 1000,
        }
    }
}

/// Frame processor state
struct ProcessorState {
    /// Input buffer
    input_buffer: Vec<u8>,
    /// Output queue
    output_queue: VecDeque<Frame>,
    /// Expected sequence number
    expected_sequence: u16,
    /// Current fragment assembly
    current_fragment: Option<FragmentAssembly>,
}

/// Fragment assembly state
struct FragmentAssembly {
    /// Frame type
    frame_type: FrameType,
    /// Sequence number
    sequence: u16,
    /// Assembled payload
    payload: Vec<u8>,
    /// Expected total size
    expected_size: Option<usize>,
}

/// Frame processor
pub struct FrameProcessor {
    /// Configuration
    config: FrameProcessorConfig,
    /// Connection states
    connection_states: std::collections::HashMap<ConnectionId, ProcessorState>,
}

impl FrameProcessor {
    /// Create a new frame processor
    pub fn new(config: FrameProcessorConfig) -> Self {
        Self {
            config,
            connection_states: std::collections::HashMap::new(),
        }
    }

    /// Process incoming data and extract frames
    pub fn process_incoming_data(
        &mut self,
        connection_id: ConnectionId,
        data: &[u8],
    ) -> Result<Vec<Frame>, FramingError> {
        let state = self
            .connection_states
            .entry(connection_id)
            .or_insert_with(|| ProcessorState {
                input_buffer: Vec::with_capacity(self.config.buffer_size),
                output_queue: VecDeque::new(),
                expected_sequence: 0,
                current_fragment: None,
            });

        // Add data to input buffer
        if state.input_buffer.len() + data.len() > self.config.buffer_size {
            return Err(FramingError::BufferOverflow);
        }

        state.input_buffer.extend_from_slice(data);

        // Extract frames
        let mut frames = Vec::new();

        while let Some(frame) = self.extract_frame_from_buffer(connection_id)? {
            frames.push(frame);
        }

        Ok(frames)
    }

    /// Extract a single frame from the input buffer
    fn extract_frame_from_buffer(
        &mut self,
        connection_id: ConnectionId,
    ) -> Result<Option<Frame>, FramingError> {
        let state = self
            .connection_states
            .get_mut(&connection_id)
            .ok_or_else(|| FramingError::Internal("Connection state not found".to_string()))?;

        // Check if we have enough data for a frame header
        if state.input_buffer.len() < 8 {
            return Ok(None);
        }

        // Parse frame header
        let frame_type_byte = state.input_buffer[0];
        let flags_byte = state.input_buffer[1];
        let sequence = u16::from_le_bytes([state.input_buffer[2], state.input_buffer[3]]);
        let payload_length = u32::from_le_bytes([
            state.input_buffer[4],
            state.input_buffer[5],
            state.input_buffer[6],
            state.input_buffer[7],
        ]) as usize;

        // Validate frame size
        if payload_length > self.config.max_frame_size {
            return Err(FramingError::FrameTooLarge(
                payload_length,
                self.config.max_frame_size,
            ));
        }

        // Check if we have the complete frame
        let total_frame_size = 8 + payload_length;
        if state.input_buffer.len() < total_frame_size {
            return Err(FramingError::IncompleteFrame(
                total_frame_size - state.input_buffer.len(),
            ));
        }

        // Parse frame type
        let frame_type = match frame_type_byte {
            0 => FrameType::Request,
            1 => FrameType::Response,
            2 => FrameType::Event,
            3 => FrameType::Error,
            4 => FrameType::Control,
            5 => FrameType::Continuation,
            _ => return Err(FramingError::InvalidFrameType(frame_type_byte)),
        };

        // Parse flags
        let flags = FrameFlags {
            fragmented: (flags_byte & 0x01) != 0,
            last_fragment: (flags_byte & 0x02) != 0,
            requires_response: (flags_byte & 0x04) != 0,
            compressed: (flags_byte & 0x08) != 0,
            encrypted: (flags_byte & 0x10) != 0,
        };

        // Extract payload
        let payload = state.input_buffer[8..total_frame_size].to_vec();

        // Remove frame from buffer
        state.input_buffer.drain(0..total_frame_size);

        // Create frame
        let frame = Frame {
            frame_type,
            sequence,
            payload,
            flags,
        };

        debug!(
            "Extracted frame: type={:?}, seq={}, size={}",
            frame.frame_type,
            frame.sequence,
            frame.payload.len()
        );

        // Handle fragmentation
        if frame.flags.fragmented {
            self.handle_fragmented_frame(connection_id, frame)
        } else {
            Ok(Some(frame))
        }
    }

    /// Handle fragmented frames
    fn handle_fragmented_frame(
        &mut self,
        connection_id: ConnectionId,
        frame: Frame,
    ) -> Result<Option<Frame>, FramingError> {
        let state = self
            .connection_states
            .get_mut(&connection_id)
            .ok_or_else(|| FramingError::Internal("Connection state not found".to_string()))?;

        if state.current_fragment.is_none() {
            // Start new fragment assembly
            state.current_fragment = Some(FragmentAssembly {
                frame_type: frame.frame_type,
                sequence: frame.sequence,
                payload: frame.payload,
                expected_size: None,
            });

            debug!("Started fragment assembly for sequence {}", frame.sequence);

            if frame.flags.last_fragment {
                // Single fragment that was marked as fragmented
                let assembly = state.current_fragment.take().unwrap();
                return Ok(Some(Frame {
                    frame_type: assembly.frame_type,
                    sequence: assembly.sequence,
                    payload: assembly.payload,
                    flags: FrameFlags {
                        fragmented: false,
                        last_fragment: true,
                        requires_response: frame.flags.requires_response,
                        compressed: frame.flags.compressed,
                        encrypted: frame.flags.encrypted,
                    },
                }));
            }
        } else {
            // Continue fragment assembly
            let assembly = state.current_fragment.as_mut().unwrap();

            if assembly.sequence != frame.sequence {
                return Err(FramingError::SequenceError(
                    assembly.sequence,
                    frame.sequence,
                ));
            }

            assembly.payload.extend_from_slice(&frame.payload);

            if frame.flags.last_fragment {
                // Complete fragment assembly
                let assembly = state.current_fragment.take().unwrap();

                debug!(
                    "Completed fragment assembly for sequence {} (total size: {})",
                    assembly.sequence,
                    assembly.payload.len()
                );

                return Ok(Some(Frame {
                    frame_type: assembly.frame_type,
                    sequence: assembly.sequence,
                    payload: assembly.payload,
                    flags: FrameFlags {
                        fragmented: false,
                        last_fragment: true,
                        requires_response: frame.flags.requires_response,
                        compressed: frame.flags.compressed,
                        encrypted: frame.flags.encrypted,
                    },
                }));
            }
        }

        Ok(None)
    }

    /// Serialize a frame to bytes
    pub fn serialize_frame(&self, frame: &Frame) -> Result<Vec<u8>, FramingError> {
        let mut buffer = Vec::with_capacity(8 + frame.payload.len());

        // Frame type
        let frame_type_byte = match frame.frame_type {
            FrameType::Request => 0,
            FrameType::Response => 1,
            FrameType::Event => 2,
            FrameType::Error => 3,
            FrameType::Control => 4,
            FrameType::Continuation => 5,
        };
        buffer.push(frame_type_byte);

        // Flags
        let mut flags_byte = 0u8;
        if frame.flags.fragmented {
            flags_byte |= 0x01;
        }
        if frame.flags.last_fragment {
            flags_byte |= 0x02;
        }
        if frame.flags.requires_response {
            flags_byte |= 0x04;
        }
        if frame.flags.compressed {
            flags_byte |= 0x08;
        }
        if frame.flags.encrypted {
            flags_byte |= 0x10;
        }
        buffer.push(flags_byte);

        // Sequence
        buffer.extend_from_slice(&frame.sequence.to_le_bytes());

        // Payload length
        buffer.extend_from_slice(&(frame.payload.len() as u32).to_le_bytes());

        // Payload
        buffer.extend_from_slice(&frame.payload);

        Ok(buffer)
    }

    /// Fragment a large frame
    pub fn fragment_frame(
        &self,
        frame: Frame,
        max_fragment_size: usize,
    ) -> Result<Vec<Frame>, FramingError> {
        if frame.payload.len() <= max_fragment_size {
            return Ok(vec![frame]);
        }

        let mut fragments = Vec::new();
        let mut offset = 0;

        while offset < frame.payload.len() {
            let end = (offset + max_fragment_size).min(frame.payload.len());
            let is_last = end == frame.payload.len();

            let fragment = Frame {
                frame_type: frame.frame_type.clone(),
                sequence: frame.sequence,
                payload: frame.payload[offset..end].to_vec(),
                flags: FrameFlags {
                    fragmented: true,
                    last_fragment: is_last,
                    requires_response: if is_last {
                        frame.flags.requires_response
                    } else {
                        false
                    },
                    compressed: frame.flags.compressed,
                    encrypted: frame.flags.encrypted,
                },
            };

            fragments.push(fragment);
            offset = end;
        }

        debug!("Fragmented frame into {} fragments", fragments.len());
        Ok(fragments)
    }

    /// Remove connection state
    pub fn remove_connection(&mut self, connection_id: ConnectionId) {
        self.connection_states.remove(&connection_id);
        debug!(
            "Removed frame processor state for connection {}",
            connection_id
        );
    }

    /// Get buffer usage for a connection
    pub fn get_buffer_usage(&self, connection_id: ConnectionId) -> Option<usize> {
        self.connection_states
            .get(&connection_id)
            .map(|state| state.input_buffer.len())
    }
}

impl Default for FrameProcessor {
    fn default() -> Self {
        Self::new(FrameProcessorConfig::default())
    }
}
