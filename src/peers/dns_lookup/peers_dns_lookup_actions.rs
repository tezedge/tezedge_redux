use std::net::SocketAddr;

use crate::Port;

use super::DnsLookupError;

#[derive(Debug, Clone)]
pub struct PeersDnsLookupInitAction {
    pub address: String,
    pub port: Port,
}

#[derive(Debug, Clone)]
pub struct PeersDnsLookupErrorAction {
    pub error: DnsLookupError,
}

#[derive(Debug, Clone)]
pub struct PeersDnsLookupSuccessAction {
    pub addresses: Vec<SocketAddr>,
}

#[derive(Debug, Clone)]
pub struct PeersDnsLookupFinishAction;
