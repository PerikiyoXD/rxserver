//! Reply Serialization
//!
//! This module handles serialization of X11 replies to wire format.

use crate::protocol::{message::Reply, X11BufMutExt};
use crate::todo_medium;
use bytes::{BufMut, BytesMut};

/// Serialize a reply to wire format
pub fn serialize_reply(reply: &Reply, sequence: u16, buf: &mut BytesMut) {
    buf.put_u8(1); // Reply type

    match reply {
        Reply::GetWindowAttributes(reply) => {
            buf.put_u8(reply.backing_store);
            buf.put_u16(sequence);
            buf.put_u32(3); // Length in 4-byte units
            buf.put_u32(reply.visual);
            buf.put_u16(reply.class as u16);
            buf.put_u8(reply.bit_gravity as u8);
            buf.put_u8(reply.win_gravity as u8);
            buf.put_u32(reply.backing_planes);
            buf.put_u32(reply.backing_pixel);
            buf.put_u8(if reply.save_under { 1 } else { 0 });
            buf.put_u8(if reply.map_is_installed { 1 } else { 0 });
            buf.put_u8(reply.map_state);
            buf.put_u8(if reply.override_redirect { 1 } else { 0 });
            buf.put_u32(reply.colormap);
            buf.put_u32(reply.all_event_masks.bits());
            buf.put_u32(reply.your_event_mask.bits());
            buf.put_u32(reply.do_not_propagate_mask.bits());
            buf.put_u32(0); // Padding
        }
        Reply::GetGeometry(reply) => {
            buf.put_u8(reply.depth);
            buf.put_u16(sequence);
            buf.put_u32(0); // Reply length (0 for fixed-length replies)
            buf.put_u32(reply.root);
            buf.put_i16(reply.x);
            buf.put_i16(reply.y);
            buf.put_u16(reply.width);
            buf.put_u16(reply.height);
            buf.put_u16(reply.border_width);
            // Pad to 32 bytes total (10 bytes of padding needed)
            buf.put_padding(22); // Current position is 22 bytes, pad to 32
        }
        Reply::InternAtom(reply) => {
            buf.put_u8(0); // Unused byte
            buf.put_u16(sequence);
            buf.put_u32(0); // Reply length (0 for fixed-length replies)
            buf.put_u32(reply.atom);
            // Pad to 32 bytes total (20 bytes of padding needed)
            buf.put_padding(12); // Current position is 12 bytes, pad to 32
        }
        Reply::GetAtomName(reply) => {
            buf.put_u8(0); // Unused byte
            buf.put_u16(sequence);
            buf.put_x11_string(&reply.name); // Use X11 string helper
            buf.put_u32(0); // Additional padding to reach reply format
        }
        Reply::GrabPointer(reply) => {
            buf.put_u8(reply.status); // Status byte
            buf.put_u16(sequence);
            buf.put_u32(0); // Reply length (0 for fixed-length replies)
                            // Pad to 32 bytes total (24 bytes of padding needed)
            buf.put_padding(8); // Current position is 8 bytes, pad to 32
        }        Reply::QueryTree(reply) => {
            buf.put_u8(0); // Unused byte
            buf.put_u16(sequence);
            let children_len = reply.children.len();
            buf.put_u32(children_len as u32); // Reply length in 4-byte units
            buf.put_u32(reply.root);
            buf.put_u32(reply.parent);
            buf.put_u16(children_len as u16);
            buf.put_u16(0); // Padding
                            // Add padding to reach 32 bytes for header
            buf.put_padding(18); // Current position is 18 bytes, pad to 32
            // Add children
            for child in &reply.children {
                buf.put_u32(*child);
            }
        }        Reply::GetProperty(reply) => {
            buf.put_u8(reply.format);
            buf.put_u16(sequence);
            let data_len = reply.data.len();
            let padded_len = (data_len + 3) & !3;
            buf.put_u32((padded_len / 4) as u32); // Reply length in 4-byte units
            buf.put_u32(reply.property_type);
            buf.put_u32(reply.bytes_after);
            buf.put_u32(data_len as u32); // Value length
                                          // Pad header to 32 bytes (12 bytes padding needed)
            buf.put_padding(20); // Current position is 20 bytes, pad to 32
            // Add property data
            buf.extend_from_slice(&reply.data);
            // Add padding to 4-byte boundary using wire utility
            buf.put_padding(data_len);
        }        _ => {
            todo_medium!(
                "reply_serialization",
                "Most reply types not implemented yet"
            );
            // Default reply structure
            buf.put_u8(0);
            buf.put_u16(sequence);
            buf.put_u32(0);
            // Pad to 32 bytes
            buf.put_padding(8); // Current position is 8 bytes, pad to 32
        }
    }
}
