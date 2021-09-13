use std::net::SocketAddr;

/// Try writing next part/message to the peer.
#[derive(Debug, Clone)]
pub struct PeerTryWriteAction {
    pub address: SocketAddr,
}
