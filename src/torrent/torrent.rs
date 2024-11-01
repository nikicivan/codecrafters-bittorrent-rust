use crate::torrent::hashes::Hashes;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Torrent {
    pub announce: String,
    pub info: Info,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Info {
    pub name: String,
    pub length: usize,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    pub pieces: Hashes,
}

impl Torrent {
    pub fn info_hash(&self) -> [u8; 20] {
        let info_bytes = serde_bencode::to_bytes(&self.info).expect("encode info into bytes");
        let mut hasher = Sha1::new();
        hasher.update(&info_bytes);
        let result = hasher.finalize();
        result.into()
    }
}
