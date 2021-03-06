use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::peer::PeerToken;
use crate::service::mio_service::PeerConnectionIncomingAcceptError;

/// Accept incoming peer connection.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConnectionIncomingAcceptAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConnectionIncomingAcceptSuccessAction {
    pub token: PeerToken,
    pub address: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConnectionIncomingAcceptErrorAction {
    pub error: PeerConnectionIncomingAcceptError,
}
