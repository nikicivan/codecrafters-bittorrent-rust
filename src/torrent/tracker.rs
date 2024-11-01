use crate::torrent::{peers::Peers, torrent::Torrent};
use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TrackerRequest {
    /// The info hash of the torrent
    // pub info_hash: [String],
    /// A unique identifier for your client
    /// 20 bytes long, will need to be URL encoded
    /// Note: this is NOT the hexadecimal representation, which is 40 bytes long
    pub peer_id: String,
    /// The port your client is listening on
    pub port: u16,
    /// The total amount uploaded so far
    pub uploaded: u64,
    /// The total amount downloaded so far
    pub downloaded: u64,
    /// whether the peer list should use the compact representation
    /// For the purposes of this challenge, set this to 1.
    /// The compact representation is more commonly used in the wild, the non-compact representation is mostly supported for backward-compatibility.
    pub compact: u8,
    pub left: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerResponse {
    /// An integer, indicating how often your client should make a request to the tracker.
    /// You can ignore this value for the purposes of this challenge.
    #[allow(unused)]
    pub interval: u64,
    /// A string, which contains list of peers that your client can connect to.
    /// Each peer is represented using 6 bytes.
    ///  The first 4 bytes are the peer's IP address
    ///  and the last 2 bytes are the peer's port number.
    pub peers: Peers,
}

impl TrackerRequest {
    pub fn new(t: &Torrent) -> Self {
        Self {
            peer_id: "-0-1-2-3-4-5-6-7-8-9".to_string(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            compact: 1,
            left: t.info.length,
        }
    }

    pub fn gen_url(&self, url: &String, info_hash: &[u8; 20]) -> Result<String> {
        let params = serde_urlencoded::to_string(&self).context("url encode request parameters")?;
        let encoded_info_hash = url_encode(info_hash);

        Ok(format!(
            "{}?{}&info_hash={}",
            url, params, encoded_info_hash
        ))
    }

    pub async fn send(&self, url: &String, info_hash: &[u8; 20]) -> Result<TrackerResponse> {
        let tracker_url = self.gen_url(url, info_hash)?;
        let response = reqwest::get(tracker_url)
            .await
            .with_context(|| format!("failed to query tracker"))?;

        let response = response
            .bytes()
            .await
            .with_context(|| format!("failed to fetch tracker response"))?;

        let response: TrackerResponse = serde_bencode::from_bytes(&response)
            .with_context(|| format!("Failed to serde response"))?;

        Ok(response)
    }
}

fn url_encode(t: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(3 * t.len());

    for &byte in t {
        encoded.push('%');
        encoded.push_str(&hex::encode(&[byte]));
    }

    encoded
}
