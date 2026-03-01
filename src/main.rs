use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with 'i', it's an integer: i<number>e
    if encoded_value.starts_with('i') {
        let end_index = encoded_value.find('e').unwrap();
        let number_string = &encoded_value[1..end_index];
        let number = number_string.parse::<i64>().unwrap();
        serde_json::Value::Number(serde_json::Number::from(number))
    }
    // If encoded_value starts with a digit, it's a string: <length>:<data>
    else if encoded_value.chars().next().unwrap().is_ascii_digit() {
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<usize>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number];
        serde_json::Value::String(string.to_string())
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

// Usage: your_program.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // TODO: Uncomment the code below to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
