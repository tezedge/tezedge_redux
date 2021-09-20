use serde::{Deserialize, Serialize};
use std::io;
use std::net::SocketAddr;

use crate::io_error_kind::IOErrorKind;
use crate::peer::PeerToken;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConnectionInitAction {
    pub address: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConnectionPendingAction {
    pub address: SocketAddr,
    pub token: PeerToken,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConnectionErrorAction {
    pub address: SocketAddr,
    pub error: IOErrorKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConnectionSuccessAction {
    pub address: SocketAddr,
}
