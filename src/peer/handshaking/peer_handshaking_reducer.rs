use redux_rs::ActionWithId;

use crate::{
    action::Action,
    peer::{connection::outgoing::PeerConnectionOutgoingState, Peer, PeerStatus},
    State,
};

use super::{PeerHandshaking, PeerHandshakingStatus};

pub fn peer_handshaking_reducer(state: &mut State, action: &ActionWithId<Action>) {
    match &action.action {
        Action::PeerHandshakingInit(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                match peer.status {
                    PeerStatus::Connecting(PeerConnectionOutgoingState::Success { token }) => {
                        peer.status = PeerStatus::Handshaking(PeerHandshaking {
                            token,
                            incoming: false,
                            status: PeerHandshakingStatus::Init,
                        });
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
