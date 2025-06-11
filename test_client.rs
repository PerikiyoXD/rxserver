use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to X11 server at 127.0.0.1:6000...");
    
    let mut stream = TcpStream::connect("127.0.0.1:6000")?;
    println!("Connected successfully!");
    
    // Create X11 connection setup request
    let setup_request = create_connection_setup();
    
    println!("Sending connection setup request ({} bytes)...", setup_request.len());
    stream.write_all(&setup_request)?;
    
    // Read the response
    let mut response = vec![0u8; 1024];
    let bytes_read = stream.read(&mut response)?;
    
    println!("Received response: {} bytes", bytes_read);
    
    if bytes_read > 0 {
        let status = response[0];
        match status {
            1 => {
                println!("✅ Connection setup SUCCESS!");
                
                // Parse some key fields from the response
                if bytes_read >= 8 {
                    let additional_length = u16::from_le_bytes([response[6], response[7]]);
                    println!("Additional data length: {} 4-byte units ({} bytes)", 
                             additional_length, additional_length * 4);
                }
                
                // Try to send a BIG-REQUESTS query (like xeyes does)
                println!("\nSending QueryExtension request for BIG-REQUESTS...");
                let query_extension = create_query_extension_request("BIG-REQUESTS");
                stream.write_all(&query_extension)?;
                
                // Read response
                let mut ext_response = vec![0u8; 32];
                let ext_bytes = stream.read(&mut ext_response)?;
                println!("QueryExtension response: {} bytes", ext_bytes);
                
                if ext_bytes > 0 {
                    println!("Response bytes: {:?}", &ext_response[..ext_bytes.min(16)]);
                }
                
                println!("✅ Client successfully communicated with server!");
            }
            0 => {
                println!("❌ Connection setup FAILED");
                if bytes_read > 1 {
                    let reason_len = response[1] as usize;
                    if bytes_read >= 8 + reason_len {
                        let reason = String::from_utf8_lossy(&response[8..8 + reason_len]);
                        println!("Reason: {}", reason);
                    }
                }
            }
            2 => {
                println!("🔐 Authentication required");
            }
            _ => {
                println!("❓ Unknown status: {}", status);
            }
        }
        
        // Show raw response for debugging
        println!("\nRaw response (first 64 bytes): {:02x?}", &response[..bytes_read.min(64)]);
    }
    
    Ok(())
}

fn create_connection_setup() -> Vec<u8> {
    let mut request = Vec::new();
    
    // Byte order (little endian)
    request.push(b'l');
    
    // Unused
    request.push(0);
    
    // Protocol version (11.0)
    request.extend_from_slice(&11u16.to_le_bytes()); // major
    request.extend_from_slice(&0u16.to_le_bytes());  // minor
    
    // Authorization protocol name length
    request.extend_from_slice(&0u16.to_le_bytes());
    
    // Authorization protocol data length  
    request.extend_from_slice(&0u16.to_le_bytes());
    
    // Unused
    request.extend_from_slice(&[0u8; 2]);
    
    request
}

fn create_query_extension_request(extension_name: &str) -> Vec<u8> {
    let mut request = Vec::new();
    
    // Opcode for QueryExtension
    request.push(98);
    
    // Unused
    request.push(0);
    
    // Request length in 4-byte units
    let name_len = extension_name.len();
    let total_len = 2 + ((name_len + 3) / 4); // 2 for header + padded name length
    request.extend_from_slice(&(total_len as u16).to_le_bytes());
    
    // Name length
    request.extend_from_slice(&(name_len as u16).to_le_bytes());
    
    // Unused
    request.extend_from_slice(&[0u8; 2]);
    
    // Extension name
    request.extend_from_slice(extension_name.as_bytes());
    
    // Pad to 4-byte boundary
    let padding = (4 - (name_len % 4)) % 4;
    request.extend_from_slice(&vec![0u8; padding]);
    
    request
}
