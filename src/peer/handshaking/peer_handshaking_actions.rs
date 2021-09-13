use std::io;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct PeerHandshakingInitAction {
    pub address: SocketAddr,
}
