use std::io;
use std::net::SocketAddr;

use tezos_messages::p2p::binary_message::BinaryChunk;

#[derive(Debug, Clone)]
pub struct PeerConnectionMessageWriteInitAction {
    pub address: SocketAddr,
    /// Encoded `ConnectionMessage`.
    pub conn_msg: BinaryChunk,
}

/// Some amount of bytes have been successfuly written.
#[derive(Debug, Clone)]
pub struct PeerConnectionMessagePartWrittenAction {
    pub address: SocketAddr,
    pub bytes_written: usize,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionMessageWriteErrorAction {
    pub address: SocketAddr,
    pub error: io::ErrorKind,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionMessageWriteSuccessAction {
    pub address: SocketAddr,
}
