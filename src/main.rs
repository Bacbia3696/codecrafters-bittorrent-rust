use std::env;

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, usize) {
    // If encoded_value starts with 'i', it's an integer: i<number>e
    if encoded_value.starts_with('i') {
        let end_index = encoded_value.find('e').unwrap();
        let number_string = &encoded_value[1..end_index];
        let number = number_string.parse::<i64>().unwrap();
        (
            serde_json::Value::Number(serde_json::Number::from(number)),
            end_index + 1,
        )
    }
    // If encoded_value starts with a digit, it's a string: <length>:<data>
    else if encoded_value.chars().next().unwrap().is_ascii_digit() {
        let colon_index = encoded_value.find(':').unwrap();
        let length_string = &encoded_value[..colon_index];
        let length = length_string.parse::<usize>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + length];
        (
            serde_json::Value::String(string.to_string()),
            colon_index + 1 + length,
        )
    }
    // If encoded_value starts with 'l', it's a list: l<elements>e
    else if encoded_value.starts_with('l') {
        let mut elements = Vec::new();
        let mut pos = 1; // Skip the 'l'

        while pos < encoded_value.len() && encoded_value.as_bytes()[pos] != b'e' {
            let (element, consumed) = decode_bencoded_value(&encoded_value[pos..]);
            elements.push(element);
            pos += consumed;
        }

        (serde_json::Value::Array(elements), pos + 1) // +1 to skip the 'e'
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

fn decode_bencoded_value_simple(encoded_value: &str) -> serde_json::Value {
    decode_bencoded_value(encoded_value).0
}

// Usage: your_program.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value_simple(encoded_value);
        println!("{}", decoded_value);
    } else {
        println!("unknown command: {}", args[1])
    }
}
