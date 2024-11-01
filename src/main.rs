use serde_json::{Map, Value};
use std::env;

fn decode_bencoded_value(encoded_value: &str) -> Value {
    decode_bencoded_start_at(encoded_value, 0).0
}

fn decode_bencoded_start_at(raw_value: &str, start_index: usize) -> (Value, usize) {
    let encoded_value = &raw_value[start_index..];

    eprintln!("index: {}, encoded_value: {}", start_index, encoded_value);

    match encoded_value.chars().next().unwrap() {
        c if c.is_digit(10) => {
            // Example: "5:hello" -> "hello
            let colon_idx = encoded_value.find(":").unwrap();
            let number_string = &encoded_value[..colon_idx];
            let number = number_string.parse::<i64>().unwrap();
            let string = &encoded_value[colon_idx + 1..colon_idx + 1 + number as usize];

            let part_len = colon_idx + number as usize;
            (
                Value::String(string.to_string()),
                start_index + part_len + 1,
            )
        }
        'i' => {
            let end_index = encoded_value.find("e").unwrap();
            let number_string = &encoded_value[1..end_index];
            let number = number_string.parse::<i64>().unwrap();
            let real_number_str = number.to_string();
            if real_number_str.len() == number_string.len() {
                let part_len = number_string.len() + 2;
                (number.into(), start_index + part_len)
            } else {
                panic!("Unhandled encoded value: {}", encoded_value)
            }
        }
        'l' => {
            let mut list: Vec<Value> = Vec::new();
            let mut idx = start_index + 1;

            while raw_value.chars().nth(idx).unwrap() != 'e' {
                let (value, new_idx) = decode_bencoded_start_at(raw_value, idx);
                list.push(value);
                idx = new_idx;

                if raw_value.len() <= idx {
                    panic!("Unhandled encoded value: {}", encoded_value)
                }
            }

            (list.into(), idx + 1)
        }
        'd' => {
            let mut dictionary: Map<String, Value> = Map::new();
            let mut idx = start_index + 1;

            while raw_value.chars().nth(idx).unwrap() != 'e' {
                let (key, new_idx) = decode_bencoded_start_at(raw_value, idx);

                let (value, new_idx) = decode_bencoded_start_at(raw_value, new_idx);

                if let Value::String(key) = key {
                    dictionary.insert(key, value);
                } else {
                    panic!("Unhandled encoded value: {}", encoded_value)
                }

                idx = new_idx;
                if raw_value.len() <= idx {
                    panic!("Unhandled encoded value: {}", encoded_value)
                }
            }

            (dictionary.into(), idx + 1)
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
