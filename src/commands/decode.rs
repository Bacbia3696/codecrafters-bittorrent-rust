use crate::bencode::{decode_bencode, print_value};

pub fn decode(encoded: &str) -> Result<(), String> {
    let value = decode_bencode(encoded.as_bytes())?;
    print_value(&value);
    println!();
    Ok(())
}
