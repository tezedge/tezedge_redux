use std::net::SocketAddr;

use crypto::crypto_box::PublicKey;
use shell_state::networking::peer::PeerCrypto;
use tezos_messages::p2p::{
    binary_message::BinaryChunk,
    encoding::{metadata::MetadataMessage, version::NetworkVersion},
};

use crate::Port;

use super::connecting::PeerConnecting;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum RequestState {
    // Idle { at: SystemTime },
    // Pending { at: SystemTime },
    // Success { at: SystemTime },
    // Error { at: SystemTime },
    Idle,
    Pending,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct ReceivedConnectionMessageData {
    port: Port,
    compatible_version: Option<NetworkVersion>,
    public_key: PublicKey,
    encoded: BinaryChunk,
}

#[derive(Debug, Clone)]
pub enum PeerHandshakingStatus {
    /// Exchange Connection message.
    ExchangeConnectionMessage {
        sent: RequestState,
        received: Option<ReceivedConnectionMessageData>,
        sent_conn_msg: BinaryChunk,
    },
    /// Exchange Metadata message.
    ExchangeMetadataMessage {
        sent: RequestState,
        received: Option<MetadataMessage>,

        port: Port,
        compatible_version: Option<NetworkVersion>,
        public_key: PublicKey,
        crypto: PeerCrypto,
    },
    /// Exchange Ack message.
    ExchangeAckMessage {
        sent: RequestState,
        received: bool,

        port: Port,
        compatible_version: Option<NetworkVersion>,
        public_key: PublicKey,
        disable_mempool: bool,
        private_node: bool,
        crypto: PeerCrypto,
    },
}

#[derive(Debug, Clone)]
pub struct PeerHandshaking {
    pub status: PeerHandshakingStatus,
    pub incoming: bool,
}

#[derive(Debug, Clone)]
pub struct PeerHandshaked {
    pub address: SocketAddr,
    pub port: Port,
    pub version: NetworkVersion,
    pub public_key: PublicKey,
    pub crypto: PeerCrypto,
    pub disable_mempool: bool,
    pub private_node: bool,
}

#[derive(Debug, Clone)]
pub enum PeerStatus {
    /// Peer is a potential peer.
    Potential,

    Connecting(PeerConnecting),
    Handshaking(PeerHandshaking),
    Handshaked(PeerHandshaked),
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub status: PeerStatus,
}
