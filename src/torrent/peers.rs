use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug, Clone)]
pub struct Peers(pub Vec<SocketAddrV4>);

struct PeersVisitor;

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = Peers;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("6 bytes, the first 4 bytes are a peer's IP address and the last 2 are a peer's port number")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() % 6 != 0 {
            return Err(E::custom(format!("Invalid length of {}", v.len())));
        }
        Ok(Peers(
            v.chunks(6)
                .map(|slice_6| {
                    let ip = Ipv4Addr::new(slice_6[0], slice_6[1], slice_6[2], slice_6[3]);
                    let port = u16::from_be_bytes([slice_6[4], slice_6[5]]);
                    SocketAddrV4::new(ip, port)
                })
                .collect(),
        ))
    }
}

impl<'de> Deserialize<'de> for Peers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

impl Serialize for Peers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut v = Vec::with_capacity(self.0.len() * 6);
        for peer in &self.0 {
            v.extend_from_slice(&peer.ip().octets());
            v.extend_from_slice(&peer.port().to_be_bytes());
        }

        serializer.serialize_bytes(&v)
    }
}

// #[cfg(test)]
// mod test {
//     use crate::torrent::torrent::Torrent;

//     use super::*;
//     use anyhow::Context;
//     use serde_bencode::from_bytes;
//     use std::fs;

//     #[test]
//     fn test_tracker_request() -> anyhow::Result<()> {
//         let f: Vec<u8> = fs::read("sample.torrent")
//             .context("read torrent file")
//             .unwrap();
//         let t: Torrent = from_bytes(&f).context("parse torrent file").unwrap();
//         let tracker_request = TrackerRequest::new(&t);
//         let url = tracker_request.gen_url(&t.announce, &t.info_hash())?;
//         // let a=tracker_request.send(&t.announce);
//         // println!("{:?}", a);
//         Ok(assert_eq!(url.as_str(),"http://bittorrent-test-tracker.codecrafters.io/announce?peer_id=-0-1-2-3-4-5-6-7-8-9&port=6881&uploaded=0&downloaded=0&compact=1&info_hash=%d6%9f%91%e6%b2%ae%4c%54%24%68%d1%07%3a%71%d4%ea%13%87%9a%7f"))
//     }
// }
