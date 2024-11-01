use crate::torrent::sign::Sign;
use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};

pub fn decode_bencoded_value(encoded_value: &str) -> Result<Value> {
    Ok(decode_bencoded_start_at(encoded_value, 0)
        .with_context(|| format!("Unable to call decode_bencoded_start_at"))?
        .0)
}

fn decode_bencoded_start_at(raw_value: &str, start_index: usize) -> Result<(Value, usize)> {
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
            Ok((
                Value::String(string.to_string()),
                (start_index + part_len + 1),
            ))
        }
        'i' => {
            let end_index = encoded_value.find("e").unwrap();
            let number_string = &encoded_value[1..end_index];
            let number = number_string.parse::<i64>().unwrap();
            let real_number_str = number.to_string();
            if real_number_str.len() == number_string.len() {
                let part_len = number_string.len() + 2;
                Ok((number.into(), start_index + part_len))
            } else {
                bail!("Unhandled encoded value: {}", encoded_value)
            }
        }
        'l' => {
            let mut list: Vec<Value> = Vec::new();
            let mut idx = start_index + 1;

            while raw_value.chars().nth(idx).unwrap() != 'e' {
                let (value, new_idx) = decode_bencoded_start_at(raw_value, idx)
                    .with_context(|| format!("Unable to decode bencoded start at"))?;

                list.push(value);
                idx = new_idx;

                if raw_value.len() <= idx {
                    bail!("Unhandled encoded value: {}", encoded_value)
                }
            }
            Ok((list.into(), (idx + 1)))
        }
        'd' => {
            let mut dictionary: Map<String, Value> = Map::new();
            let mut idx = start_index + 1;

            while raw_value.chars().nth(idx).unwrap() != 'e' {
                let (key, new_idx) = decode_bencoded_start_at(raw_value, idx)
                    .with_context(|| format!("Unable to decode bencoded start at"))?;

                let (value, new_idx) = decode_bencoded_start_at(raw_value, new_idx)
                    .with_context(|| format!("Unable to decode bencoded start at"))?;

                if let Value::String(key) = key {
                    dictionary.insert(key, value);
                } else {
                    bail!("Unhandled encoded value: {}", encoded_value)
                }

                idx = new_idx;
                if raw_value.len() <= idx {
                    bail!("Unhandled encoded value: {}", encoded_value)
                }
            }

            Ok((dictionary.into(), (idx + 1)))
        }
        _ => {
            bail!("Unhandled encoded value: {}", encoded_value)
        }
    }
}

pub fn decode_bencoded_vec(encoded_vec: &Vec<u8>) -> Result<Value> {
    Ok(decode_bencoded_vec_start_at(&encoded_vec, 0)
        .with_context(|| format!("Unable to call decode_bencoded_vec_start_at function"))?
        .0)
}

pub fn decode_bencoded_vec_start_at(raw_vec: &[u8], start_index: usize) -> Result<(Value, usize)> {
    let encoded_vec = &raw_vec[start_index..];

    match encoded_vec.iter().next().unwrap() {
        &n if 48 <= n && n <= 57 => {
            let colon_index = encoded_vec.iter().position(|&x| x == Sign::COLON).unwrap();
            match read_vec_u8_to_string(&encoded_vec[0..colon_index]) {
                Some(number_string) => {
                    let number = number_string.parse::<i64>().unwrap();
                    let read_result: Option<String> = read_vec_u8_to_string(
                        &encoded_vec[colon_index + 1..colon_index + 1 + number as usize],
                    );
                    if let Some(string) = read_result {
                        let part_len = colon_index + number as usize;
                        Ok((Value::String(string), (start_index + part_len + 1)))
                    } else {
                        Ok((
                            encoded_vec[colon_index + 1..colon_index + 1 + number as usize]
                                .to_vec()
                                .into(),
                            (start_index + colon_index + number as usize + 1),
                        ))
                    }
                }
                _ => {
                    bail!(
                        "Can not read length of the string at index: {}",
                        start_index
                    );
                }
            }
        }
        &Sign::I => {
            let end_index = encoded_vec.iter().position(|&x| x == Sign::E).unwrap();
            let number_string = read_vec_u8_to_string(&encoded_vec[1..end_index]).unwrap();
            let number = number_string.parse::<i64>().unwrap();
            let real_number_str = number.to_string();

            if real_number_str.len() == number_string.len() {
                let part_len = number_string.len() + 2;
                Ok((number.into(), (start_index + part_len)))
            } else {
                bail!("Unhandled encoded value at index: {}", start_index);
            }
        }
        &Sign::L => {
            let mut list: Vec<Value> = Vec::new();
            let mut index = start_index + 1;

            while raw_vec.len() > index && raw_vec[index] != Sign::E {
                let (value, new_index) = decode_bencoded_vec_start_at(&raw_vec, index)
                    .with_context(|| format!("Unable to bencode vector from start index"))?;

                list.push(value);
                index = new_index;
            }
            Ok((list.into(), (index + 1)))
        }
        &Sign::D => {
            let mut dict: Map<String, Value> = Map::new();
            let mut index = start_index + 1;

            while raw_vec.len() > index && raw_vec[index] != Sign::E {
                let (key, new_index) = decode_bencoded_vec_start_at(raw_vec, index)
                    .with_context(|| format!("Unable to bencode vector from start index"))?;

                let (value, new_index) = decode_bencoded_vec_start_at(raw_vec, new_index)
                    .with_context(|| format!("Unable to bencode vector from start index"))?;

                if let Value::String(key) = key {
                    dict.insert(key, value);
                } else {
                    bail!("Unhandled encoded value at: {}", start_index)
                }
                index = new_index;
            }
            Ok((dict.into(), (index + 1)))
        }
        _ => {
            bail!("Unhandled encoded value at: {}", start_index)
        }
    }
}

fn read_vec_u8_to_string(vec: &[u8]) -> Option<String> {
    match String::from_utf8(vec.to_vec()) {
        Ok(result) => Some(result),
        Err(_) => None,
    }
}
