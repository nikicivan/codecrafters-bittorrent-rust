use clap::{command, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Command {
    Decode { encoded_value: String },
    Info { torrent: PathBuf },
    Peers { torrent: PathBuf },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
