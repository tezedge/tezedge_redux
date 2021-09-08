use std::net::SocketAddr;

use crate::Port;

#[derive(Debug, Clone)]
pub enum PeersDnsLookupStatus {
    Init,
    Success { addresses: Vec<SocketAddr> },
    Error { error: dns_lookup::LookupErrorKind },
}

#[derive(Debug, Clone)]
pub struct PeersDnsLookupState {
    pub address: String,
    pub port: Port,
    pub status: PeersDnsLookupStatus,
}
