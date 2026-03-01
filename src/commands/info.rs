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

    let length = value
        .get("info")
        .and_then(|info| info.get("length"))
        .and_then(|v| v.as_integer())
        .ok_or("Missing 'info.length' key")?;

    // Get the raw bytes of the info dictionary and compute SHA-1 hash
    let info = value.get("info").ok_or("Missing 'info' key")?;
    let info_bytes = info.raw_bytes();
    let hash = Sha1::digest(info_bytes);
    let info_hash = hex::encode(hash);

    println!("Tracker URL: {}", announce);
    println!("Length: {}", length);
    println!("Info Hash: {}", info_hash);
    Ok(())
}
