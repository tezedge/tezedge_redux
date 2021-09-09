use std::io;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct PeerConnectionInitAction {
    pub address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionPendingAction {
    pub address: SocketAddr,
    pub token: mio::Token,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionErrorAction {
    pub address: SocketAddr,
    pub error: io::ErrorKind,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionSuccessAction {
    pub address: SocketAddr,
}
