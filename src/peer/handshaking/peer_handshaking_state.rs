use crypto::crypto_box::PublicKey;
use shell_state::networking::peer::PeerCrypto;
use tezos_messages::p2p::{
    binary_message::BinaryChunk,
    encoding::{metadata::MetadataMessage, version::NetworkVersion},
};

use crate::Port;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MessageSendState {
    Idle,
    Pending { written: usize },
    Success,
    // TODO: use custom error instead.
    Error { error: std::io::ErrorKind },
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
        /// Encoded `ConnectionMessage` to be sent.
        send_conn_msg: BinaryChunk,
        sent: MessageSendState,
        received: Option<ReceivedConnectionMessageData>,
    },
    /// Exchange Metadata message.
    ExchangeMetadataMessage {
        /// Encoded `MetadataMessage` to be sent.
        send_meta_msg: BinaryChunk,
        sent: MessageSendState,
        received: Option<MetadataMessage>,

        port: Port,
        compatible_version: Option<NetworkVersion>,
        public_key: PublicKey,
        crypto: PeerCrypto,
    },
    /// Exchange Ack message.
    ExchangeAckMessage {
        /// Encoded `AckMessage` to be sent.
        send_ack_message: BinaryChunk,
        sent: MessageSendState,
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
    pub token: PeerToken,
    pub status: PeerHandshakingStatus,
    pub incoming: bool,
}
