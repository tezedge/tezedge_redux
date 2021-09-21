use redux_rs::ActionWithId;

use crate::{
    action::Action,
    peer::{Peer, PeerStatus},
    State,
};

use super::PeerConnecting;

pub fn peer_connecting_reducer(state: &mut State, action: &ActionWithId<Action>) {
    match &action.action {
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
                    peer.status = PeerStatus::Connecting(PeerConnecting::Pending {
                        token: action.token,
                    });
                }
            }
        }
        Action::PeerConnectionError(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                if matches!(
                    peer.status,
                    PeerStatus::Connecting(PeerConnecting::Idle)
                        | PeerStatus::Connecting(PeerConnecting::Pending { .. })
                ) {
                    peer.status = PeerStatus::Connecting(PeerConnecting::Error {
                        error: action.error,
                    });
                }
            }
        }
        Action::PeerConnectionSuccess(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                if let PeerStatus::Connecting(PeerConnecting::Pending { token }) = peer.status {
                    peer.status = PeerStatus::Connecting(PeerConnecting::Success { token });
                }
            }
        }
        _ => {}
    }
}
