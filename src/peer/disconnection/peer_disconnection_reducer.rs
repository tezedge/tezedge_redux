use redux_rs::ActionWithId;

use crate::{
    action::Action,
    peer::{
        connecting::PeerConnecting,
        disconnection::{PeerDisconnecting, PeerDisconnectionState},
        Peer, PeerStatus,
    },
    State,
};

pub fn peer_disconnection_reducer(state: &mut State, action: &ActionWithId<Action>) {
    match &action.action {
        Action::PeerDisconnect(action) => {
            let peer = match state.peers.get_mut(&action.address) {
                Some(v) => v,
                None => return,
            };

            peer.status = match &peer.status {
                PeerStatus::Connecting(state) => match state {
                    PeerConnecting::Pending { token } => PeerDisconnecting {
                        token: token.clone(),
                    }
                    .into(),
                    PeerConnecting::Success { token } => PeerDisconnecting {
                        token: token.clone(),
                    }
                    .into(),
                    _ => return,
                },
                PeerStatus::Handshaking(state) => PeerDisconnecting { token: state.token }.into(),
                PeerStatus::Handshaked(state) => PeerDisconnecting { token: state.token }.into(),
                _ => return,
            };
        }
        Action::PeerDisconnected(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                peer.status = match &peer.status {
                    PeerStatus::Potential => return,
                    PeerStatus::Connecting(_) => PeerStatus::Disconnected,
                    PeerStatus::Handshaking(_) => PeerStatus::Disconnected,
                    PeerStatus::Handshaked(_) => PeerStatus::Disconnected,
                    PeerStatus::Disconnecting(_) => PeerStatus::Disconnected,
                    PeerStatus::Disconnected => return,
                };
            }
        }
        _ => {}
    }
}
