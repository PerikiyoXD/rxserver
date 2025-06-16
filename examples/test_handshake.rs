//! Test X11 handshake with RXServer
//! 
//! This example connects to RXServer and performs the X11 connection setup handshake

use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing X11 handshake with RXServer...");
    
    // Connect to the server
    println!("📡 Connecting to 127.0.0.1:6000...");
    let mut stream = TcpStream::connect("127.0.0.1:6000")?;
    println!("✅ Connected!");
    
    // Create X11 connection setup request according to the specification
    let mut setup_request = Vec::new();
    
    // Byte 0: byte-order (0x6c = LSB first, 0x42 = MSB first)
    setup_request.push(0x6c); // LSB first
    
    // Byte 1: unused
    setup_request.push(0);
    
    // Bytes 2-3: protocol-major-version (CARD16) - should be 11
    setup_request.extend_from_slice(&11u16.to_le_bytes());
    
    // Bytes 4-5: protocol-minor-version (CARD16) - should be 0
    setup_request.extend_from_slice(&0u16.to_le_bytes());
    
    // Bytes 6-7: length of authorization-protocol-name (n)
    setup_request.extend_from_slice(&0u16.to_le_bytes()); // No auth for simplicity
    
    // Bytes 8-9: length of authorization-protocol-data (d)
    setup_request.extend_from_slice(&0u16.to_le_bytes()); // No auth data
    
    // Bytes 10-11: unused
    setup_request.extend_from_slice(&0u16.to_le_bytes());
    
    // No authorization name or data since n=0 and d=0
    
    println!("📤 Sending connection setup request ({} bytes):", setup_request.len());
    println!("   Byte order: 0x{:02x} ({})", setup_request[0], 
             if setup_request[0] == 0x6c { "LSB first" } else { "MSB first" });
    println!("   Protocol version: {}.{}", 11, 0);
    println!("   Auth name length: 0");
    println!("   Auth data length: 0");
    
    // Send the request
    stream.write_all(&setup_request)?;
    stream.flush()?;
    
    // Read the response
    let mut response_buffer = vec![0u8; 2048]; // Large buffer for complete response
    let bytes_read = stream.read(&mut response_buffer)?;
    response_buffer.truncate(bytes_read);
    
    println!("📥 Received {} bytes from server", bytes_read);
    
    if response_buffer.is_empty() {
        println!("❌ No response received!");
        return Ok(());
    }
    
    // Parse the response according to X11 specification
    let status = response_buffer[0];
    
    match status {
        1 => {
            println!("🎉 SUCCESS! Connection accepted");
            
            if response_buffer.len() < 8 {
                println!("❌ Response too short for success format");
                return Ok(());
            }
            
            // Parse success response structure
            let _unused = response_buffer[1];
            let protocol_major = u16::from_le_bytes([response_buffer[2], response_buffer[3]]);
            let protocol_minor = u16::from_le_bytes([response_buffer[4], response_buffer[5]]);
            let additional_data_len = u16::from_le_bytes([response_buffer[6], response_buffer[7]]);
            
            println!("📋 Response details:");
            println!("   Protocol version: {}.{}", protocol_major, protocol_minor);
            println!("   Additional data length: {} (4-byte units)", additional_data_len);
            println!("   Expected additional bytes: {}", additional_data_len * 4);
            println!("   Actual response length: {} bytes", response_buffer.len());
            
            // Verify we have all the additional data
            let expected_total_len = 8 + (additional_data_len as usize * 4);
            if response_buffer.len() >= expected_total_len {
                println!("✅ Received complete server information!");
                
                // Parse some server info if we have enough data
                if response_buffer.len() >= 32 {
                    let release_number = u32::from_le_bytes([
                        response_buffer[8], response_buffer[9],
                        response_buffer[10], response_buffer[11]
                    ]);
                    let resource_id_base = u32::from_le_bytes([
                        response_buffer[12], response_buffer[13],
                        response_buffer[14], response_buffer[15]
                    ]);
                    let resource_id_mask = u32::from_le_bytes([
                        response_buffer[16], response_buffer[17],
                        response_buffer[18], response_buffer[19]
                    ]);
                    
                    println!("🔍 Server information:");
                    println!("   Release number: {}", release_number);
                    println!("   Resource ID base: 0x{:08x}", resource_id_base);
                    println!("   Resource ID mask: 0x{:08x}", resource_id_mask);
                }
            } else {
                println!("⚠️  Response appears incomplete (expected {}, got {})", 
                         expected_total_len, response_buffer.len());
            }
        }
        0 => {
            println!("❌ FAILED! Connection rejected");
            if response_buffer.len() > 1 {
                let reason_len = response_buffer[1];
                println!("   Reason length: {} bytes", reason_len);
                
                if response_buffer.len() >= 8 {
                    let protocol_major = u16::from_le_bytes([response_buffer[2], response_buffer[3]]);
                    let protocol_minor = u16::from_le_bytes([response_buffer[4], response_buffer[5]]);
                    let additional_data_len = u16::from_le_bytes([response_buffer[6], response_buffer[7]]);
                    
                    println!("   Protocol version: {}.{}", protocol_major, protocol_minor);
                    println!("   Additional data length: {} (4-byte units)", additional_data_len);
                    
                    // Try to extract the reason string
                    let reason_start = 8;
                    if response_buffer.len() > reason_start && reason_len > 0 {
                        let reason_end = (reason_start + reason_len as usize).min(response_buffer.len());
                        let reason = String::from_utf8_lossy(&response_buffer[reason_start..reason_end]);
                        println!("   Reason: \"{}\"", reason);
                    }
                }
            }
        }
        2 => {
            println!("🔐 Authentication required");
            if response_buffer.len() >= 8 {
                let additional_data_len = u16::from_le_bytes([response_buffer[6], response_buffer[7]]);
                println!("   Additional data length: {} (4-byte units)", additional_data_len);
            }
        }
        _ => {
            println!("❓ Unknown response status: {}", status);
        }
    }
    
    // Print hex dump for debugging
    println!("\n🔍 Response hex dump (first 64 bytes):");
    for (i, chunk) in response_buffer.chunks(16).take(4).enumerate() {
        print!("  {:04x}: ", i * 16);
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02x} ", byte);
            if j == 7 { print!(" "); } // Add space in middle for readability
        }
        
        // Pad the line if it's shorter than 16 bytes
        for _ in chunk.len()..16 {
            print!("   ");
        }
        
        print!(" |");
        for byte in chunk {
            let c = if byte.is_ascii_graphic() || *byte == b' ' { 
                *byte as char 
            } else { 
                '.' 
            };
            print!("{}", c);
        }
        println!("|");
    }
    
    if response_buffer.len() > 64 {
        println!("  ... ({} more bytes)", response_buffer.len() - 64);
    }
    
    println!("\n✨ Test completed successfully!");
    
    Ok(())
}
