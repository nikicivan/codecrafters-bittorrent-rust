use anyhow::{Context, Result};
use std::env;
use torrent::{
    metainfo_reader::read_file_to_bytes,
    parser::{decode_bencoded_value, decode_bencoded_vec},
};

mod torrent;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value)
                .with_context(|| format!("Unable to decode value"))?;

            println!("{}", decoded_value.to_string());
        }
        "info" => {
            let metainfo_file_path = &args[2];
            let metainfo_file_content = read_file_to_bytes(metainfo_file_path)
                .with_context(|| format!("Unable to read metainfo file content"))?;

            let parsed_value = decode_bencoded_vec(&metainfo_file_content)
                .with_context(|| format!("Unable to parse value"))?;

            println!(
                "Tracker URL: {}",
                parsed_value["announce"].as_str().unwrap().trim_matches('"')
            );

            println!(
                "Length: {:?}",
                parsed_value["info"]["length"].as_i64().unwrap()
            );
        }
        _ => {
            println!("unknown command: {}", args[1]);
        }
    }

    Ok(())
}
