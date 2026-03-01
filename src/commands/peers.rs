use crate::bencode::decode_bencode;
use sha1::{Digest, Sha1};
use std::fs;

pub fn peers(torrent_path: &str) -> Result<(), String> {
    let data = fs::read(torrent_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let value = decode_bencode(&data)?;

    let announce = value
        .get("announce")
        .and_then(|v| v.as_string())
        .ok_or("Missing 'announce' key")?;

    let info = value.get("info").ok_or("Missing 'info' key")?;

    let length = info
        .get("length")
        .and_then(|v| v.as_integer())
        .ok_or("Missing 'info.length' key")?;

    // Get the raw bytes of the info dictionary and compute SHA-1 hash
    let info_bytes = info.raw_bytes();
    let hash = Sha1::digest(info_bytes);

    // Build tracker URL with query parameters
    let peer_id = generate_peer_id();
    let url = build_tracker_url(announce, &hash, &peer_id, length)?;

    // Make request to tracker
    let response =
        reqwest::blocking::get(&url).map_err(|e| format!("Failed to contact tracker: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Tracker returned status: {}", response.status()));
    }

    let response_bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Parse the bencoded response
    let response_value = decode_bencode(&response_bytes)?;

    // Extract peer list from compact format
    let peers_data = response_value
        .get("peers")
        .and_then(|v| v.as_bytes())
        .ok_or("Missing 'peers' in tracker response")?;

    // Compact format: each peer is 6 bytes (4 bytes IP + 2 bytes port)
    let peers = parse_compact_peers(peers_data)?;

    // Print each peer as IP:PORT
    for peer in peers {
        println!("{}", peer);
    }

    Ok(())
}

/// Generate a unique 20-byte peer ID
/// Format: -<2 char client ID><4 char version><12 random chars> = 20 bytes total
fn generate_peer_id() -> String {
    // Using a standard BitTorrent peer ID format
    // -BR0001 followed by 12 random alphanumeric chars
    "-BR0001-123456789012".to_string()
}

/// Build the tracker URL with all required query parameters
fn build_tracker_url(
    announce: &str,
    info_hash: &[u8],
    peer_id: &str,
    left: i64,
) -> Result<String, String> {
    // URL encode the info_hash (binary data, 20 bytes)
    let info_hash_encoded = url_encode_binary(info_hash);

    // URL encode the peer_id
    let peer_id_encoded = url_encode_string(peer_id);

    let url = format!(
        "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
        announce, info_hash_encoded, peer_id_encoded, left
    );

    Ok(url)
}

/// URL encode binary data (percent encoding for non-alphanumeric bytes)
fn url_encode_binary(data: &[u8]) -> String {
    data.iter()
        .flat_map(|&b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.') {
                vec![b as char]
            } else {
                format!("%{:02X}", b).chars().collect()
            }
        })
        .collect()
}

/// URL encode a string (percent encoding for special characters)
fn url_encode_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                vec![c]
            } else {
                format!("%{:02X}", c as u8).chars().collect()
            }
        })
        .collect()
}

/// Parse compact peer format: 4 bytes IP + 2 bytes port, repeated
fn parse_compact_peers(data: &[u8]) -> Result<Vec<String>, String> {
    const PEER_SIZE: usize = 6; // 4 bytes IP + 2 bytes port

    if !data.len().is_multiple_of(PEER_SIZE) {
        return Err(format!(
            "Invalid peers data length: {} (not divisible by 6)",
            data.len()
        ));
    }

    let mut peers = Vec::new();

    for chunk in data.chunks(PEER_SIZE) {
        let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);
        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
        peers.push(format!("{}:{}", ip, port));
    }

    Ok(peers)
}
