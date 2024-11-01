use anyhow::{bail, Context, Result};
use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
use std::{env, fs};
use torrent::{
    metainfo_reader::{self},
    parser::{self, decode_bencoded_value},
};

mod torrent;

// #[derive(Debug, Clone, Deserialize)]
// struct Torrent {
//     announce: String,
//     info: Info,
// }

// #[derive(Debug, Clone, Deserialize)]
// struct Info {
//     name: String,
//     /// For the purposes of transfer, files are split into fixed-size pieces which are all the same
//     /// length except for possibly the last one which may be truncated. piece length is almost
//     /// always a power of two, most commonly 2^18 = 256K (BitTorrent prior to version 3.2 uses 2
//     /// 20 = 1 M as default).
//     #[serde(rename = "piece_length")]
//     plength: usize,
//     /// Each entry of `pieces` is the SHA1 hash of the piece at the corresponding index.
//     pieces: Vec<u8>,
//     #[serde(flatten)]
//     keys: Keys,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(untagged)]
// enum Keys {
//     /// If `length` is present then the download represents a single file.
//     SingleFile {
//         /// The length of the file in bytes.
//         length: usize,
//     },
//     /// Otherwise it represents a set of files which go in a directory structure.
//     ///
//     /// For the purposes of the other keys in `Info`, the multi-file case is treated as only having
//     /// a single file by concatenating the files in the order they appear in the files list.
//     MultiFile { files: Vec<File> },
// }

// #[derive(Debug, Clone, Deserialize)]
// struct File {
//     /// The length of the file, in bytes.
//     length: usize,
//     /// Subdirectory names for this file, the last of which is the actual file name
//     /// (a zero length list is an error case).
//     path: Vec<String>,
// }

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

            let info = serde_bencode::from_bytes::<Value>(&fs::read(metainfo_file_path)?)?;

            if let Value::Dict(dict) = info {
                let announce = dict
                    .get(b"announce".as_ref())
                    .with_context(|| format!("no announce"))?;

                let info = dict
                    .get(b"info".as_ref())
                    .with_context(|| format!("no info"))?;

                let hash: String = hex::encode(Sha1::digest(serde_bencode::to_bytes(info)?));

                if let (Value::Bytes(announce), Value::Dict(info)) = (announce, info) {
                    println!("Tracker URL: {}", String::from_utf8_lossy(announce));
                    let length = info.get(b"length".as_ref()).context("no length")?;

                    if let Value::Int(length) = length {
                        println!("Length: {length}");
                        println!("Info Hash: {hash}");
                    } else {
                        bail!("Invalid torrent file")
                    }
                } else {
                    bail!("Invalid torrent file")
                }
            } else {
                bail!("Invalid torrent file")
            }

            let metainfo_file_path = &args[2];
            let metainfo_file_content = metainfo_reader::read_file_to_bytes(metainfo_file_path)
                .with_context(|| format!("Failed to read metainfo file"))?;

            let parsed_value = parser::decode_bencoded_vec(&metainfo_file_content)
                .with_context(|| format!("Failed to parse value"))?;

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
