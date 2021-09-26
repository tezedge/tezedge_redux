use redux_rs::{ActionWithId, Store};

use crate::peer::handshaking::PeerHandshakingInitAction;
use crate::peer::PeerStatus;
use crate::peers::remove::PeersRemoveAction;
use crate::service::{MioService, Service};
use crate::{action::Action, State};

use super::PeerDisconnectedAction;

pub fn peer_disconnection_effects<S>(
    store: &mut Store<State, S, Action>,
    action: &ActionWithId<Action>,
) where
    S: Service,
{
    match &action.action {
        Action::PeerDisconnect(action) => {
            let address = action.address;
            let peer = match store.state.get().peers.get(&address) {
                Some(v) => v,
                None => return,
            };

            let peer_token = match &peer.status {
                PeerStatus::Disconnecting(disconnection_state) => disconnection_state.token,
                _ => return,
            };

            store.service().mio().peer_disconnect(peer_token);

            store.dispatch(PeerDisconnectedAction { address }.into());
        }
        Action::PeerDisconnected(action) => {
            if let Some(peer) = store.state.get().peers.get(&action.address) {
                if matches!(&peer.status, PeerStatus::Disconnected) {
                    let address = action.address;

                    store.dispatch(PeersRemoveAction { address }.into());
                }
            }
        }
        _ => {}
    }
}
