use crate::bencode::decode_bencode;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const BLOCK_SIZE: u32 = 16 * 1024; // 16 KiB

// Message IDs
const MSG_CHOKE: u8 = 0;
#[allow(dead_code)]
const MSG_UNCHOKE: u8 = 1;
#[allow(dead_code)]
const MSG_INTERESTED: u8 = 2;
#[allow(dead_code)]
const MSG_NOT_INTERESTED: u8 = 3;
#[allow(dead_code)]
const MSG_HAVE: u8 = 4;
const MSG_BITFIELD: u8 = 5;
const MSG_REQUEST: u8 = 6;
const MSG_PIECE: u8 = 7;
#[allow(dead_code)]
const MSG_CANCEL: u8 = 8;

pub fn download(torrent_path: &str, output_path: &str) -> Result<(), String> {
    // Parse torrent file
    let data = fs::read(torrent_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let value = decode_bencode(&data)?;

    let announce = value
        .get("announce")
        .and_then(|v| v.as_string())
        .ok_or("Missing 'announce' key")?;

    let info = value.get("info").ok_or("Missing 'info' key")?;

    let file_length = info
        .get("length")
        .and_then(|v| v.as_integer())
        .ok_or("Missing 'info.length' key")? as u64;

    let piece_length = info
        .get("piece length")
        .and_then(|v| v.as_integer())
        .ok_or("Missing 'info.piece length' key")? as u32;

    let pieces = info
        .get("pieces")
        .and_then(|v| v.as_bytes())
        .ok_or("Missing 'info.pieces' key")?;

    // Get the raw bytes of the info dictionary and compute SHA-1 hash
    let info_bytes = info.raw_bytes();
    let info_hash = Sha1::digest(info_bytes);

    // Calculate number of pieces
    let num_pieces = file_length.div_ceil(piece_length as u64);
    println!("File size: {} bytes", file_length);
    println!("Piece length: {} bytes", piece_length);
    println!("Number of pieces: {}", num_pieces);

    // Get peers from tracker
    let peers = get_peers(announce, &info_hash, file_length)?;
    if peers.is_empty() {
        return Err("No peers available".to_string());
    }
    println!("Got {} peers from tracker", peers.len());

    // Download all pieces
    let mut all_piece_data = Vec::with_capacity(file_length as usize);

    for piece_index in 0..num_pieces {
        println!("Downloading piece {}/{}...", piece_index, num_pieces - 1);

        // Calculate this piece's size
        let this_piece_size = if piece_index == num_pieces - 1 {
            // Last piece
            (file_length % piece_length as u64) as u32
        } else {
            piece_length
        };

        // Get the expected hash for this piece
        let piece_hash = get_piece_hash(pieces, piece_index as u32)?;

        // Try to download from available peers
        let mut piece_data = None;
        for peer in &peers {
            match download_piece_from_peer(
                peer,
                &info_hash,
                piece_index as u32,
                this_piece_size,
                &piece_hash,
            ) {
                Ok(data) => {
                    piece_data = Some(data);
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "Failed to download piece {} from {}: {}",
                        piece_index, peer, e
                    );
                    continue;
                }
            }
        }

        match piece_data {
            Some(data) => {
                all_piece_data.extend_from_slice(&data);
                println!("  ✓ Piece {} downloaded and verified", piece_index);
            }
            None => {
                return Err(format!(
                    "Failed to download piece {} from any peer",
                    piece_index
                ));
            }
        }
    }

    // Verify total size
    if all_piece_data.len() != file_length as usize {
        return Err(format!(
            "Downloaded size mismatch: expected {}, got {}",
            file_length,
            all_piece_data.len()
        ));
    }

    // Write to output file
    let mut file =
        File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    file.write_all(&all_piece_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    println!("Download complete! File saved to {}", output_path);
    Ok(())
}

/// Get the SHA-1 hash for a specific piece
fn get_piece_hash(pieces: &[u8], piece_index: u32) -> Result<[u8; 20], String> {
    let start = piece_index as usize * 20;
    let end = start + 20;

    if end > pieces.len() {
        return Err(format!(
            "Piece index {} out of range (have {} bytes of piece hashes)",
            piece_index,
            pieces.len()
        ));
    }

    let mut hash = [0u8; 20];
    hash.copy_from_slice(&pieces[start..end]);
    Ok(hash)
}

/// Get list of peers from tracker
fn get_peers(announce: &str, info_hash: &[u8], left: u64) -> Result<Vec<String>, String> {
    let peer_id = generate_peer_id();
    let url = build_tracker_url(announce, info_hash, &peer_id, left)?;

    let response =
        reqwest::blocking::get(&url).map_err(|e| format!("Failed to contact tracker: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Tracker returned status: {}", response.status()));
    }

    let response_bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let response_value = decode_bencode(&response_bytes)?;

    // Check for failure message
    if let Some(failure) = response_value
        .get("failure reason")
        .and_then(|v| v.as_string())
    {
        return Err(format!("Tracker error: {}", failure));
    }

    let peers_data = response_value
        .get("peers")
        .and_then(|v| v.as_bytes())
        .ok_or("Missing 'peers' in tracker response")?;

    parse_compact_peers(peers_data)
}

/// Build tracker URL
fn build_tracker_url(
    announce: &str,
    info_hash: &[u8],
    peer_id: &str,
    left: u64,
) -> Result<String, String> {
    let info_hash_encoded = url_encode_binary(info_hash);
    let peer_id_encoded = url_encode_string(peer_id);

    Ok(format!(
        "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
        announce, info_hash_encoded, peer_id_encoded, left
    ))
}

/// URL encode binary data
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

/// URL encode string
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

/// Generate peer ID
fn generate_peer_id() -> String {
    "-BR0001-123456789012".to_string()
}

/// Parse compact peers format
fn parse_compact_peers(data: &[u8]) -> Result<Vec<String>, String> {
    const PEER_SIZE: usize = 6;

    if !data.len().is_multiple_of(PEER_SIZE) {
        return Err(format!("Invalid peers data length: {}", data.len()));
    }

    Ok(data
        .chunks(PEER_SIZE)
        .map(|chunk| {
            let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            format!("{}:{}", ip, port)
        })
        .collect())
}

/// Download a piece from a specific peer
fn download_piece_from_peer(
    peer: &str,
    info_hash: &[u8],
    piece_index: u32,
    piece_size: u32,
    expected_hash: &[u8; 20],
) -> Result<Vec<u8>, String> {
    // Connect to peer
    let mut stream = TcpStream::connect(peer).map_err(|e| format!("Failed to connect: {}", e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    // Send handshake
    let our_peer_id = generate_peer_id_bytes();
    let handshake = build_handshake(info_hash, &our_peer_id);
    stream
        .write_all(&handshake)
        .map_err(|e| format!("Failed to send handshake: {}", e))?;

    // Read handshake response
    let mut response = vec![0u8; 68];
    stream
        .read_exact(&mut response)
        .map_err(|e| format!("Failed to read handshake: {}", e))?;

    parse_handshake_response(&response, info_hash)?;

    // Read bitfield (message id 5)
    let (msg_len, msg_id) = read_message_header(&mut stream)?;
    if msg_id != MSG_BITFIELD {
        return Err(format!("Expected bitfield (5), got {}", msg_id));
    }

    // Read and ignore bitfield payload
    let mut bitfield = vec![0u8; msg_len as usize];
    stream
        .read_exact(&mut bitfield)
        .map_err(|e| format!("Failed to read bitfield: {}", e))?;

    // Send interested (message id 2)
    send_message(&mut stream, MSG_INTERESTED, &[])?;

    // Wait for unchoke (message id 1)
    loop {
        let (msg_len, msg_id) = read_message_header(&mut stream)?;

        // Skip payload
        if msg_len > 0 {
            let mut payload = vec![0u8; msg_len as usize];
            stream
                .read_exact(&mut payload)
                .map_err(|e| format!("Failed to read payload: {}", e))?;
        }

        if msg_id == MSG_UNCHOKE {
            break;
        } else if msg_id == MSG_CHOKE {
            return Err("Peer choked".to_string());
        }
    }

    // Calculate blocks needed for this piece
    let mut piece_data = vec![0u8; piece_size as usize];
    let mut offset: u32 = 0;

    while offset < piece_size {
        let block_size = std::cmp::min(BLOCK_SIZE, piece_size - offset);

        // Send request (message id 6)
        // Payload: piece_index (4 bytes), begin (4 bytes), length (4 bytes)
        let mut request_payload = Vec::with_capacity(12);
        request_payload.extend_from_slice(&piece_index.to_be_bytes());
        request_payload.extend_from_slice(&offset.to_be_bytes());
        request_payload.extend_from_slice(&block_size.to_be_bytes());

        send_message(&mut stream, MSG_REQUEST, &request_payload)?;

        offset += block_size;
    }

    // Receive piece messages
    offset = 0;
    while offset < piece_size {
        let block_size = std::cmp::min(BLOCK_SIZE, piece_size - offset);

        let (msg_len, msg_id) = read_message_header(&mut stream)?;

        if msg_id != MSG_PIECE {
            return Err(format!("Expected piece (7), got {}", msg_id));
        }

        // Read piece payload: index (4), begin (4), block (variable)
        let mut piece_payload = vec![0u8; msg_len as usize];
        stream
            .read_exact(&mut piece_payload)
            .map_err(|e| format!("Failed to read piece: {}", e))?;

        // Parse piece payload
        if piece_payload.len() < 8 {
            return Err("Piece payload too short".to_string());
        }

        let begin = u32::from_be_bytes([
            piece_payload[4],
            piece_payload[5],
            piece_payload[6],
            piece_payload[7],
        ]);
        let block_data = &piece_payload[8..];

        if block_data.len() != block_size as usize {
            return Err(format!(
                "Block size mismatch: expected {}, got {}",
                block_size,
                block_data.len()
            ));
        }

        // Copy block data to piece buffer
        let start = begin as usize;
        let end = start + block_data.len();
        piece_data[start..end].copy_from_slice(block_data);

        offset += block_size;
    }

    // Verify hash
    let actual_hash = Sha1::digest(&piece_data);
    if actual_hash.as_slice() != expected_hash {
        return Err(format!(
            "Hash mismatch! Expected: {}, Got: {}",
            hex::encode(expected_hash),
            hex::encode(actual_hash)
        ));
    }

    Ok(piece_data)
}

/// Generate peer ID bytes
fn generate_peer_id_bytes() -> [u8; 20] {
    let mut peer_id = [0u8; 20];
    peer_id[0..8].copy_from_slice(b"-BR0001-");
    (8..20).for_each(|i| {
        peer_id[i] = (i as u8) + 1;
    });
    peer_id
}

/// Build handshake message
fn build_handshake(info_hash: &[u8], peer_id: &[u8; 20]) -> Vec<u8> {
    let mut handshake = Vec::with_capacity(68);
    handshake.push(19);
    handshake.extend_from_slice(b"BitTorrent protocol");
    handshake.extend_from_slice(&[0u8; 8]);
    handshake.extend_from_slice(info_hash);
    handshake.extend_from_slice(peer_id);
    handshake
}

/// Parse handshake response
fn parse_handshake_response(response: &[u8], expected_info_hash: &[u8]) -> Result<(), String> {
    if response.len() < 68 {
        return Err(format!("Handshake too short: {}", response.len()));
    }

    let protocol_len = response[0] as usize;
    if protocol_len != 19 {
        return Err(format!("Invalid protocol length: {}", protocol_len));
    }

    let protocol = &response[1..20];
    if protocol != b"BitTorrent protocol" {
        return Err("Invalid protocol string".to_string());
    }

    let received_info_hash = &response[28..48];
    if received_info_hash != expected_info_hash {
        return Err("Info hash mismatch".to_string());
    }

    Ok(())
}

/// Read message header: length (4 bytes) + id (1 byte)
fn read_message_header(stream: &mut TcpStream) -> Result<(u32, u8), String> {
    let mut len_bytes = [0u8; 4];
    stream
        .read_exact(&mut len_bytes)
        .map_err(|e| format!("Failed to read message length: {}", e))?;

    let msg_len = u32::from_be_bytes(len_bytes);

    if msg_len == 0 {
        return Ok((0, 0)); // Keep-alive message
    }

    let mut id_bytes = [0u8; 1];
    stream
        .read_exact(&mut id_bytes)
        .map_err(|e| format!("Failed to read message id: {}", e))?;

    Ok((msg_len - 1, id_bytes[0])) // Subtract 1 for the id byte
}

/// Send a message: length (4 bytes) + id (1 byte) + payload
fn send_message(stream: &mut TcpStream, msg_id: u8, payload: &[u8]) -> Result<(), String> {
    let msg_len = (payload.len() + 1) as u32; // +1 for message id

    let mut msg = Vec::with_capacity(4 + 1 + payload.len());
    msg.extend_from_slice(&msg_len.to_be_bytes());
    msg.push(msg_id);
    msg.extend_from_slice(payload);

    stream
        .write_all(&msg)
        .map_err(|e| format!("Failed to send message: {}", e))?;

    Ok(())
}
