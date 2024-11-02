use regex::bytes;

#[derive(Debug)]
pub struct Handshake {
    pub length: u8,
    pub bittorrent: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            length: 19,
            bittorrent: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }

    pub fn to_bytes(&self) -> [u8; 68] {
        let mut bytes = [0; 68];
        bytes[0] = self.length;
        bytes[1..20].copy_from_slice(&self.bittorrent);
        bytes[28..48].copy_from_slice(&self.info_hash);
        bytes[48..68].copy_from_slice(&self.peer_id);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            length: bytes[0],
            bittorrent: bytes[1..20].try_into().unwrap(),
            reserved: bytes[20..28].try_into().unwrap(),
            info_hash: bytes[28..48].try_into().unwrap(),
            peer_id: bytes[48..68].try_into().unwrap(),
        }
    }
}
