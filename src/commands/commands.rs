use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};
use url::Url;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
#[clap(rename_all = "snake_case")]
pub enum Command {
    Decode {
        value: String,
    },
    Info {
        torrent: PathBuf,
    },
    Peers {
        torrent: PathBuf,
    },
    Handshake {
        torrent: PathBuf,
        peer_address: SocketAddr,
    },
    DownloadPiece {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
        piece: usize,
    },
    Download {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
    },
    MagnetParse {
        magnet_link: Url,
    },
    MagnetHandshake {
        magnet_link: Url,
    },
    MagnetInfo {
        magnet_link: Url,
    },
    MagnetDownloadPiece {
        #[arg(short)]
        output: PathBuf,
        magnet_link: Url,
        piece: usize,
    },
    MagnetDownload {
        #[arg(short)]
        output: PathBuf,
        magnet_link: Url,
    },
}
