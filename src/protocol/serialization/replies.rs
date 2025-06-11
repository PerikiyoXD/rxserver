//! Reply Serialization
//!
//! This module handles serialization of X11 replies to wire format.

use bytes::{BufMut, BytesMut};

use crate::{
    protocol::{serialization::wire::X11BufMutExt, Reply},
    todo_medium,
};

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
            buf.put_padding(22); // Pad to 32 bytes total
        }
        Reply::InternAtom(reply) => {
            buf.put_u8(0); // Unused byte
            buf.put_u16(sequence);
            buf.put_u32(0); // Reply length (0 for fixed-length replies)
            buf.put_u32(reply.atom);
            buf.put_padding(12); // Pad to 32 bytes total
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
            buf.put_padding(8); // Pad to 32 bytes total
        }
        Reply::QueryTree(reply) => {
            buf.put_u8(0); // Unused byte
            buf.put_u16(sequence);
            let children_len = reply.children.len();
            buf.put_u32(children_len as u32); // Reply length in 4-byte units
            buf.put_u32(reply.root);
            buf.put_u32(reply.parent);
            buf.put_u16(children_len as u16);
            buf.put_u16(0); // Padding
            buf.put_padding(18); // Pad to 32 bytes for header
            for child in &reply.children {
                buf.put_u32(*child);
            }
        }
        Reply::GetProperty(reply) => {
            buf.put_u8(reply.format);
            buf.put_u16(sequence);
            let data_len = reply.data.len();
            let padded_len = (data_len + 3) & !3;
            buf.put_u32((padded_len / 4) as u32); // Reply length in 4-byte units
            buf.put_u32(reply.property_type);
            buf.put_u32(reply.bytes_after);
            buf.put_u32(data_len as u32); // Value length
            buf.put_padding(20); // Pad header to 32 bytes
            buf.extend_from_slice(&reply.data);
            buf.put_padding(data_len); // Pad to 4-byte boundary
        }
        Reply::QueryExtension(reply) => {
            buf.put_u8(if reply.present { 1 } else { 0 }); // Present flag
            buf.put_u16(sequence);
            buf.put_u32(0); // Reply length (0 for fixed-length replies)
            buf.put_u8(reply.major_opcode);
            buf.put_u8(reply.first_event);
            buf.put_u8(reply.first_error);
            buf.put_padding(21); // Pad to 32 bytes total
        }
        _ => {
            todo_medium!(
                "serialize_reply",
                "Serialization for reply type {:?} not implemented",
                reply
            );
            buf.put_u8(0);
            buf.put_u16(sequence);
            buf.put_u32(0);
            buf.put_padding(8); // Pad to 32 bytes
        }
    }
}
