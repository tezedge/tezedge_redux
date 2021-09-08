use std::io;

#[derive(Debug, Clone)]
pub enum PeerConnecting {
    Idle,
    Pending,
    Success,
    Error { error: io::ErrorKind },
}
