use crate::torrent::{
    extension::{ExtensionHeader, ExtensionMessage, ExtensionMessageType},
    torrent::Info,
};
use anyhow::{ensure, Context, Result};
use bitvec::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{mem, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
    task::JoinSet,
};

const BLOCK_SIZE: u32 = 16 * 1024; // 16 KiB
const EXTENSION_SUPPORT_FLAG: u64 = 1 << 20;

#[derive(Serialize, Deserialize)]
pub struct Handshake {
    pub length: u8,
    pub protocol: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20]) -> Self {
        let mut reserved = 0;
        reserved |= EXTENSION_SUPPORT_FLAG;
        let peer_id: [u8; 20] = Peer::gen_peer_id().as_bytes().try_into().unwrap();
        Self {
            length: 19,
            protocol: *b"BitTorrent protocol",
            reserved: reserved.to_be_bytes(),
            info_hash,
            peer_id,
        }
    }

    pub fn supports_extension(&self) -> bool {
        self.reserved[5] & 0x10 != 0
    }
}

#[derive(Clone)]
pub struct Peer {
    pub address: SocketAddr,
    pub id: [u8; 20],
    pub stream: Arc<Mutex<TcpStream>>,
    pub supports_extension: bool,
    pub metadata_extension_id: Option<u8>,
}

impl Peer {
    pub async fn new(address: SocketAddr, info_hash: [u8; 20]) -> Result<Self> {
        let mut handshake = Handshake::new(info_hash);
        let mut handshake_bytes = bincode::serialize(&handshake)?;

        let mut peer_stream = TcpStream::connect(address)
            .await
            .context("failed to connect to peer")?;
        peer_stream
            .write_all(&handshake_bytes)
            .await
            .context("failed to send handshake")?;
        peer_stream
            .read_exact(&mut handshake_bytes)
            .await
            .context("failed to receive handshake")?;

        handshake = bincode::deserialize(&handshake_bytes)?;
        let peer = Peer {
            address,
            id: handshake.peer_id,
            stream: Arc::new(Mutex::new(peer_stream)),
            supports_extension: handshake.supports_extension(),
            metadata_extension_id: None,
        };
        Ok(peer)
    }

    pub async fn extension_handshake(&mut self) -> Result<()> {
        let ext_header = ExtensionHeader::new();
        let mut payload = serde_bencode::to_bytes(&ext_header)?;
        payload.insert(0, 0);

        let handshake = Message::new(MessageId::EXTENSION, payload);
        self.send(handshake).await?;
        let reply = self.recv().await?;
        let ext_header = serde_bencode::from_bytes::<ExtensionHeader>(&reply.payload[1..])?;
        self.metadata_extension_id = Some(ext_header.m.ut_metadata);
        Ok(())
    }

    pub async fn extension_metadata(&mut self) -> Result<Info> {
        let ext_msg = ExtensionMessage {
            msg_type: ExtensionMessageType::Request,
            piece: 0,
            total_size: None,
        };
        let mut payload = serde_bencode::to_bytes(&ext_msg)?;
        let extension_msg_id = self
            .metadata_extension_id
            .expect("metadata extension id should be set during handshake");
        payload.insert(0, extension_msg_id);

        let msg = Message::new(MessageId::EXTENSION, payload);
        self.send(msg).await?;
        let reply = self.recv().await?;
        let ext_msg = serde_bencode::from_bytes::<ExtensionMessage>(&reply.payload[1..])?;
        let metadata_piece_len = ext_msg.total_size.unwrap();
        let metadata = &reply.payload[reply.payload.len() - metadata_piece_len as usize..];
        let torrent_info = serde_bencode::from_bytes::<Info>(metadata)?;
        Ok(torrent_info)
    }

    async fn recv(&mut self) -> Result<Message> {
        let mut stream = self.stream.lock().await;
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await?;
        let length = u32::from_be_bytes(buf);

        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
        let id: MessageId = unsafe { mem::transmute(buf[0]) };

        let mut buf = vec![0u8; length as usize - mem::size_of::<MessageId>()];
        stream.read_exact(&mut buf).await?;
        Ok(Message {
            length,
            id,
            payload: buf,
        })
    }

    async fn send(&mut self, msg: Message) -> Result<()> {
        let mut stream = self.stream.lock().await;
        stream.write_all(&msg.as_bytes()).await?;
        Ok(())
    }

    pub async fn get_pieces(&mut self) -> Result<Vec<usize>> {
        let msg = self.recv().await?;
        ensure!(msg.id == MessageId::BITFIELD);
        let bitfield = BitVec::<u8, Msb0>::from_vec(msg.payload);
        let pieces = bitfield.iter_ones().collect();
        Ok(pieces)
    }

    pub async fn prepare_download(&mut self) -> Result<()> {
        let interested = Message::new(MessageId::INTERESTED, vec![]);
        self.send(interested).await?;
        let msg = self.recv().await?;
        ensure!(msg.id == MessageId::UNCHOKE);
        Ok(())
    }

    pub async fn load_piece(&mut self, index: u32, piece_len: u32) -> Result<Vec<u8>> {
        let mut piece = vec![0u8; piece_len as usize];
        let mut join_set = JoinSet::new();

        let spawn = |join_set: &mut JoinSet<_>, mut peer: Peer, offset: u32| {
            let length = BLOCK_SIZE.min(piece_len - offset);
            join_set.spawn(async move {
                match peer.load_block(index, offset, length).await {
                    Ok(msg) => (offset, msg.payload[8..].to_vec()),
                    Err(err) => {
                        eprintln!("Error loading block: {}. Will retry...", err);
                        (offset, vec![])
                    }
                }
            });
        };

        for offset in (0..piece_len).step_by(BLOCK_SIZE as usize) {
            spawn(&mut join_set, self.clone(), offset);
        }

        while let Some(join_result) = join_set.join_next().await {
            let (offset, data) = join_result.context("Task panicked")?;
            if data.is_empty() {
                spawn(&mut join_set, self.clone(), offset);
            } else {
                let start = offset as usize;
                let end = start + data.len();
                piece[start..end].copy_from_slice(&data);
            }
        }

        Ok(piece)
    }

    async fn load_block(&mut self, index: u32, begin: u32, length: u32) -> Result<Message> {
        let payload = vec![
            index.to_be_bytes(),
            begin.to_be_bytes(),
            length.to_be_bytes(),
        ]
        .concat();
        let request = Message::new(MessageId::REQUEST, payload);
        self.send(request).await?;
        let msg = self.recv().await?;
        ensure!(msg.id == MessageId::PIECE);
        Ok(msg)
    }

    pub fn gen_peer_id() -> String {
        let peer_id_len = 20;
        (0..peer_id_len)
            .map(|_| rand::thread_rng().gen_range(0..10).to_string())
            .collect()
    }
}

#[derive(Debug)]
struct Message {
    length: u32,
    id: MessageId,
    payload: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
enum MessageId {
    BITFIELD = 5,
    INTERESTED = 2,
    UNCHOKE = 1,
    REQUEST = 6,
    PIECE = 7,
    EXTENSION = 20,
}

impl Message {
    fn new(id: MessageId, payload: Vec<u8>) -> Self {
        let length = (mem::size_of::<MessageId>() + payload.len()) as u32;
        Self {
            length,
            id,
            payload,
        }
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.length.to_be_bytes());
        bytes.push(self.id as u8);
        bytes.extend(self.payload.as_slice());
        bytes
    }
}
