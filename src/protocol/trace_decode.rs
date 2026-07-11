//! Human-readable decoding of raw X11 wire bytes for trace logging.
//!
//! Every debugging session on this server has involved manually decoding
//! raw byte dumps from `server_trace.log` by hand (Python one-liners, or
//! worse, mental arithmetic) to check sequence numbers, reply lengths, or
//! event codes. This module does that decoding inline, at trace level, so
//! the log itself is legible without a side calculation - see
//! `.agents/memories/protocol.md` for the wire layouts this decodes.

use crate::protocol::ByteOrder;

/// Decode the first bytes of a server-to-client reply or event as human
/// text, for trace logging. Handles the two 32-byte-header shapes every
/// reply/event on this server uses: replies (`1, unused, sequence(2),
/// length(4), ...`) and core events (`code, detail, sequence(2), ...`).
/// Returns a short one-line summary; never panics on short/malformed input,
/// since this exists purely for diagnostics and must never be the reason a
/// trace-level log call fails.
pub fn describe_outgoing(bytes: &[u8], byte_order: ByteOrder) -> String {
    if bytes.len() < 4 {
        return format!("<{} bytes, too short to decode>", bytes.len());
    }

    let read_u16 = |lo: u8, hi: u8| -> u16 {
        match byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([lo, hi]),
            ByteOrder::BigEndian => u16::from_be_bytes([lo, hi]),
        }
    };

    let first = bytes[0];
    let second = bytes[1];
    let sequence = read_u16(bytes[2], bytes[3]);

    match first {
        0 => format!(
            "Error(code={}, sequence={})",
            second, sequence
        ),
        1 => {
            let reply_length = if bytes.len() >= 8 {
                match byte_order {
                    ByteOrder::LittleEndian => {
                        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
                    }
                    ByteOrder::BigEndian => {
                        u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
                    }
                }
            } else {
                0
            };
            format!(
                "Reply(sequence={}, extra_length_words={}, total_bytes={})",
                sequence,
                reply_length,
                32 + reply_length as usize * 4
            )
        }
        code => format!(
            "Event(code={}, detail={}, sequence={})",
            code, second, sequence
        ),
    }
}
