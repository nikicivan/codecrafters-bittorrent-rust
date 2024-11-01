use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    match encoded_value.chars().next().unwrap() {
        c if c.is_digit(10) => {
            // Example: "5:hello" -> "hello
            let colon_idx = encoded_value.find(":").unwrap();
            let number_string = &encoded_value[..colon_idx];
            let number = number_string.parse::<i64>().unwrap();
            let string = &encoded_value[colon_idx + 1..colon_idx + 1 + number as usize];
            serde_json::Value::String(string.to_string())
        }
        'i' => {
            let end_index = encoded_value.find("e").unwrap();
            let number_string = &encoded_value[1..end_index];
            let number = number_string.parse::<i64>().unwrap();
            let real_number_str = number.to_string();
            if real_number_str.len() == number_string.len() {
                number.into()
            } else {
                panic!("Unhandled encoded value: {}", encoded_value)
            }
        }
        _ => {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
