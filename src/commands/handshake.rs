use crate::bencode::decode_bencode;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub fn handshake(torrent_path: &str, peer: &str) -> Result<(), String> {
    // Parse torrent file to get info hash
    let data = fs::read(torrent_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let value = decode_bencode(&data)?;

    let info = value.get("info").ok_or("Missing 'info' key")?;

    // Get the raw bytes of the info dictionary and compute SHA-1 hash
    let info_bytes = info.raw_bytes();
    let hash = Sha1::digest(info_bytes);

    // Generate our peer ID (20 random bytes)
    let our_peer_id = generate_peer_id_bytes();

    // Connect to the peer
    let mut stream =
        TcpStream::connect(peer).map_err(|e| format!("Failed to connect to {}: {}", peer, e))?;

    // Set timeout for read operations
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    // Build and send handshake
    let handshake = build_handshake(&hash, &our_peer_id);
    stream
        .write_all(&handshake)
        .map_err(|e| format!("Failed to send handshake: {}", e))?;

    // Read response handshake
    let mut response = vec![0u8; 68]; // Expected response size
    stream
        .read_exact(&mut response)
        .map_err(|e| format!("Failed to read handshake response: {}", e))?;

    // Parse the response handshake
    let peer_id = parse_handshake_response(&response, &hash)?;

    // Print peer ID in hexadecimal format
    println!("Peer ID: {}", hex::encode(peer_id));

    Ok(())
}

/// Generate a random 20-byte peer ID
fn generate_peer_id_bytes() -> [u8; 20] {
    // Using a simple pattern for reproducibility
    // In production, you'd use a proper RNG
    let mut peer_id = [0u8; 20];

    // Use "-BR0001-" prefix (8 bytes) + 12 random bytes
    peer_id[0..8].copy_from_slice(b"-BR0001-");

    // Fill remaining 12 bytes with a pattern (in production, use random)
    (8..20).for_each(|i| {
        peer_id[i] = (i as u8) + 1;
    });

    peer_id
}

/// Build the handshake message
/// Format:
/// - 1 byte: length of protocol string (19)
/// - 19 bytes: "BitTorrent protocol"
/// - 8 bytes: reserved (zeros)
/// - 20 bytes: info hash
/// - 20 bytes: peer ID
fn build_handshake(info_hash: &[u8], peer_id: &[u8; 20]) -> Vec<u8> {
    let mut handshake = Vec::with_capacity(68);

    // Protocol string length (1 byte)
    handshake.push(19);

    // Protocol string (19 bytes)
    handshake.extend_from_slice(b"BitTorrent protocol");

    // Reserved bytes (8 bytes, all zeros)
    handshake.extend_from_slice(&[0u8; 8]);

    // Info hash (20 bytes)
    handshake.extend_from_slice(info_hash);

    // Peer ID (20 bytes)
    handshake.extend_from_slice(peer_id);

    handshake
}

/// Parse the handshake response and validate it
fn parse_handshake_response(
    response: &[u8],
    expected_info_hash: &[u8],
) -> Result<[u8; 20], String> {
    if response.len() < 68 {
        return Err(format!(
            "Handshake response too short: {} bytes (expected at least 68)",
            response.len()
        ));
    }

    // Parse protocol string length
    let protocol_len = response[0] as usize;
    if protocol_len != 19 {
        return Err(format!(
            "Invalid protocol string length: {} (expected 19)",
            protocol_len
        ));
    }

    // Parse protocol string
    let protocol = &response[1..20];
    if protocol != b"BitTorrent protocol" {
        return Err(format!(
            "Invalid protocol string: {:?}",
            String::from_utf8_lossy(protocol)
        ));
    }

    // Skip reserved bytes (8 bytes)
    // let reserved = &response[20..28];

    // Parse info hash (20 bytes)
    let received_info_hash = &response[28..48];
    if received_info_hash != expected_info_hash {
        return Err("Info hash mismatch - peer doesn't have this torrent".to_string());
    }

    // Parse peer ID (20 bytes)
    let mut peer_id = [0u8; 20];
    peer_id.copy_from_slice(&response[48..68]);

    Ok(peer_id)
}
