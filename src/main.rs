use anyhow::{Context, Result};
use clap::Parser;
use commands::commands::{Args, Command};
use std::fs;
use torrent::{
    parser::decode_bencoded_value,
    torrent::Torrent,
    tracker::{TrackerRequest, TrackerResponse},
};

mod commands;
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
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Decode { encoded_value } => {
            let decoded_value = decode_bencoded_value(&encoded_value)
                .with_context(|| format!("Failed to decode value"))?;
            println!("{}", decoded_value.to_string())
        }
        Command::Info { torrent } => {
            let f: Vec<u8> = fs::read(torrent)?;
            let t: Torrent = serde_bencode::from_bytes(&f)?;
            println!("Tracker URL: {}", t.announce);

            let length = t.info.length;
            println!("Length: {length}");

            let info_hash = t.info_hash();
            println!("Info Hash: {}", hex::encode(&info_hash));
            println!("Piece Length: {}", t.info.piece_length);
            println!("Piece Hashes:");

            for hash in t.info.pieces.0 {
                println!("{}", hex::encode(&hash));
            }
        }
        Command::Peers { torrent } => {
            let f: Vec<u8> = fs::read(torrent)?;
            let t: Torrent = serde_bencode::from_bytes(&f)?;
            let tracker = TrackerRequest::new(&t);

            let response: TrackerResponse = tracker.send(&t.announce, &t.info_hash()).await?;

            for peer in response.peers.0 {
                println!("{}", peer);
            }
        }
    }

    Ok(())
}
