use crate::bencode::decode_bencode;
use sha1::{Digest, Sha1};
use std::fs;

pub fn info(torrent_path: &str) -> Result<(), String> {
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
    let info_hash = hex::encode(hash);

    // Get piece length and piece hashes
    let piece_length = info
        .get("piece length")
        .and_then(|v| v.as_integer())
        .ok_or("Missing 'info.piece length' key")?;

    let pieces = info
        .get("pieces")
        .and_then(|v| v.as_bytes())
        .ok_or("Missing 'info.pieces' key")?;

    // Parse piece hashes (each hash is 20 bytes)
    let piece_hashes: Vec<String> = pieces.chunks(20).map(hex::encode).collect();

    println!("Tracker URL: {}", announce);
    println!("Length: {}", length);
    println!("Info Hash: {}", info_hash);
    println!("Piece Length: {}", piece_length);
    println!("Number of pieces: {}", piece_hashes.len());
    println!("Piece hashes:");
    for (i, hash) in piece_hashes.iter().enumerate() {
        println!("  {}: {}", i, hash);
    }
    Ok(())
}
