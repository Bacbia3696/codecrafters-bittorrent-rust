use std::fs;
use crate::bencode::decode_bencode;

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

    println!("Tracker URL: {}", announce);
    println!("Length: {}", length);
    Ok(())
}
