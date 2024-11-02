use anyhow::Result;
use serde_bencode::{from_str, value::Value as BencodedValue};
use serde_json::Value;

pub fn decode_bencoded_value(encoded_value: &str) -> Result<Value> {
    let value = from_str(encoded_value)?;
    let decoded = bencode_to_json(value)?;

    Ok(decoded)
}

fn bencode_to_json(value: BencodedValue) -> Result<Value> {
    match value {
        BencodedValue::Bytes(b) => Ok(Value::String(String::from_utf8(b)?)),
        BencodedValue::Int(i) => Ok(Value::Number(serde_json::Number::from(i))),
        BencodedValue::List(l) => {
            let json_list = l
                .into_iter()
                .map(|v| bencode_to_json(v))
                .collect::<Result<Vec<Value>>>()?;

            Ok(Value::Array(json_list))
        }
        BencodedValue::Dict(d) => {
            let json_map = d
                .into_iter()
                .map(|(k, v)| Ok((String::from_utf8(k)?, bencode_to_json(v)?)))
                .collect::<Result<serde_json::Map<String, Value>>>()?;
            Ok(Value::Object(json_map))
        }
    }
}
