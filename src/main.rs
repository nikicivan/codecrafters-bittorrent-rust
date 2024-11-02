use anyhow::{Context, Result};
use clap::Parser;
use commands::commands::{Args, Command};
use std::fs;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use torrent::{
    parser::decode_bencoded_value,
    peer::Handshake,
    torrent::Torrent,
    tracker::{TrackerRequest, TrackerResponse},
};

mod commands;
mod torrent;

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
        Command::Handshake { torrent, addr } => {
            let f: Vec<u8> = fs::read(torrent).context("read torrent file")?;
            let t: Torrent = serde_bencode::from_bytes(&f).context("parse torrent file")?;

            let info_hash = t.info_hash();
            let mut handshake = Handshake::new(info_hash, *b"-0-1-2-3-4-5-6-7-8-9");
            let mut stream = TcpStream::connect(&addr).await?;
            let bytes = handshake.to_bytes();
            stream.write_all(&bytes).await?;

            let mut buff = [0; 68];
            stream.read_exact(&mut buff).await?;
            handshake = Handshake::from_bytes(&buff);

            println!("Peer ID: {}", hex::encode(&handshake.peer_id));
        }
    }

    Ok(())
}
