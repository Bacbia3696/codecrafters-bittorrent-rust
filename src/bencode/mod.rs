mod value;
mod parser;

pub use value::BencodeValue;
pub use parser::decode_bencode;

/// Print a BencodeValue as JSON-like output
pub fn print_value(value: &BencodeValue) {
    match value {
        BencodeValue::Integer(n) => print!("{}", n),
        BencodeValue::String(bytes) => {
            match std::str::from_utf8(bytes) {
                Ok(s) => print!("\"{}\"", s),
                Err(_) => print!("\"<binary:{}>\"", hex::encode(bytes)),
            }
        }
        BencodeValue::List(elements) => {
            print!("[");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    print!(",");
                }
                print_value(elem);
            }
            print!("]");
        }
        BencodeValue::Dictionary(entries) => {
            print!("{{");
            for (i, (key, value)) in entries.iter().enumerate() {
                if i > 0 {
                    print!(",");
                }
                print!("\"{}\":", key);
                print_value(value);
            }
            print!("}}");
        }
    }
}
