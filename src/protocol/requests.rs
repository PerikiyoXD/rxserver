//! X11 request handling
//!
//! This module handles parsing and processing of X11 requests from clients.

use bytes::Bytes;
use crate::{Error, Result};
use super::types::*;

/// X11 request header structure
#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub opcode: u8,
    pub data: u8,
    pub length: u16,
}

/// Parsed X11 request
#[derive(Debug, Clone)]
pub enum Request {
    CreateWindow {
        depth: u8,
        wid: Window,
        parent: Window,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        class: WindowClass,
        visual: u32,
        value_mask: u32,
        value_list: Vec<u32>,
    },
    DestroyWindow {
        window: Window,
    },
    MapWindow {
        window: Window,
    },
    UnmapWindow {
        window: Window,
    },
    ConfigureWindow {
        window: Window,
        value_mask: u16,
        value_list: Vec<u32>,
    },    GetWindowAttributes {
        window: Window,
    },
    ClearArea {
        exposures: bool,
        window: Window,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    },
    // TODO: Add more request types as needed
    Unknown {
        opcode: u8,
        data: Bytes,
    },
}

/// X11 request parser
pub struct RequestParser;

impl RequestParser {
    /// Parse a request from raw bytes
    pub fn parse(data: &[u8]) -> Result<Request> {
        if data.len() < 4 {
            return Err(Error::Protocol("Request too short".to_string()));
        }

        let header = RequestHeader {
            opcode: data[0],
            data: data[1],
            length: u16::from_ne_bytes([data[2], data[3]]),
        };

        // TODO: Implement actual request parsing based on opcode
        match header.opcode {
            opcodes::CREATE_WINDOW => {
                // TODO: Parse CreateWindow request
                Ok(Request::CreateWindow {
                    depth: header.data,
                    wid: 0, // TODO: Parse from data
                    parent: 0,
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                    border_width: 0,
                    class: WindowClass::InputOutput,
                    visual: 0,
                    value_mask: 0,
                    value_list: Vec::new(),
                })
            }
            opcodes::DESTROY_WINDOW => {
                Ok(Request::DestroyWindow { window: 0 })
            }
            opcodes::MAP_WINDOW => {
                Ok(Request::MapWindow { window: 0 })
            }
            _ => {
                Ok(Request::Unknown {
                    opcode: header.opcode,
                    data: Bytes::copy_from_slice(data),
                })
            }
        }
    }
}
