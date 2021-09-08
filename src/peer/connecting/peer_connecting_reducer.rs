use crate::{
    action::Action,
    peer::{Peer, PeerStatus},
    State,
};

use super::PeerConnecting;

pub fn peer_connecting_reducer(state: &State, action: &Action) -> State {
    let mut state = state.clone();
    match action {
        Action::PeerConnectionInit(action) => {
            let peer = state.peers.entry(action.address).or_insert_with(|| Peer {
                status: PeerStatus::Potential,
            });
            if matches!(peer.status, PeerStatus::Potential) {
                peer.status = PeerStatus::Connecting(PeerConnecting::Idle);
            }
        }
        Action::PeerConnectionPending(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                if matches!(peer.status, PeerStatus::Connecting(PeerConnecting::Idle)) {
                    peer.status = PeerStatus::Connecting(PeerConnecting::Pending);
                }
            }
        }
        Action::PeerConnectionError(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                if matches!(
                    peer.status,
                    PeerStatus::Connecting(PeerConnecting::Idle)
                        | PeerStatus::Connecting(PeerConnecting::Pending)
                ) {
                    peer.status = PeerStatus::Connecting(PeerConnecting::Error {
                        error: action.error,
                    });
                }
            }
        }
        Action::PeerConnectionSuccess(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                if matches!(peer.status, PeerStatus::Connecting(PeerConnecting::Pending)) {
                    peer.status = PeerStatus::Connecting(PeerConnecting::Success);
                }
            }
            let peer = state.peers.entry(action.address).or_insert_with(|| Peer {
                status: PeerStatus::Potential,
            });
            if matches!(peer.status, PeerStatus::Potential) {
                peer.status = PeerStatus::Connecting(PeerConnecting::Idle);
            }
        }
        _ => {}
    }
    state
}
