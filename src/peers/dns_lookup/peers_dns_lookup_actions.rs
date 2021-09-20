use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::Port;

use super::DnsLookupError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeersDnsLookupInitAction {
    pub address: String,
    pub port: Port,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeersDnsLookupErrorAction {
    pub error: DnsLookupError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeersDnsLookupSuccessAction {
    pub addresses: Vec<SocketAddr>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeersDnsLookupFinishAction;
