use std::io;
use std::net::SocketAddr;

use tezos_messages::p2p::binary_message::BinaryChunk;

use crate::io_error_kind::IOErrorKind;

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
    pub error: IOErrorKind,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionMessageWriteSuccessAction {
    pub address: SocketAddr,
}
