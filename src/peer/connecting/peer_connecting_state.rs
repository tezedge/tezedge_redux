use std::io;

#[derive(Debug, Clone)]
pub enum PeerConnecting {
    Idle,
    Pending { token: mio::Token },
    Success { token: mio::Token },
    Error { error: io::ErrorKind },
}
