use serde::{Deserialize, Serialize};

use tezos_encoding::binary_reader::BinaryReaderError;

use crate::peer::chunk::read::peer_chunk_read_state::{
    PeerChunkRead, PeerChunkReadError, ReadCrypto,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerBinaryMessageReadError {
    Chunk(PeerChunkReadError),
    Decode(String),
}

impl From<BinaryReaderError> for PeerBinaryMessageReadError {
    fn from(error: BinaryReaderError) -> Self {
        Self::Decode(error.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerBinaryMessageReadState {
    Init {
        crypto: ReadCrypto,
    },
    PendingFirstChunk {
        chunk: PeerChunkRead,
    },
    Pending {
        buffer: Vec<u8>,
        size: usize,
        chunk: PeerChunkRead,
    },
    Ready {
        crypto: ReadCrypto,
        message: Vec<u8>,
    },
    Error {
        error: PeerBinaryMessageReadError,
    },
}
