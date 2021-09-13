use crypto::crypto_box::PublicKey;
use shell_state::networking::peer::PeerCrypto;
use tezos_messages::p2p::encoding::version::NetworkVersion;

use crate::Port;

use super::{connecting::PeerConnecting, handshaking::PeerHandshaking};

#[derive(Debug, Clone)]
pub struct PeerHandshaked {
    pub token: mio::Token,
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
