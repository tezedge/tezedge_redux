use serde::{Deserialize, Serialize};
use std::io;

use crate::io_error_kind::IOErrorKind;
use crate::peer::PeerToken;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerConnectionOutgoingState {
    Idle,
    Pending { token: PeerToken },
    Success { token: PeerToken },
    Error { error: IOErrorKind },
}
