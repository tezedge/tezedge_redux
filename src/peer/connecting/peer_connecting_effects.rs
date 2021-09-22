use redux_rs::{ActionWithId, Store};

use crate::peer::handshaking::PeerHandshakingInitAction;
use crate::peer::PeerStatus;
use crate::service::{MioService, Service};
use crate::{action::Action, State};

use super::{
    PeerConnecting, PeerConnectionErrorAction, PeerConnectionPendingAction,
    PeerConnectionSuccessAction,
};

pub fn peer_connecting_effects<S>(
    store: &mut Store<State, S, Action>,
    action: &ActionWithId<Action>,
) where
    S: Service,
{
    match &action.action {
        Action::PeerConnectionInit(action) => {
            let address = action.address;
            let result = store.service().mio().peer_connection_init(address);
            store.dispatch(match result {
                Ok(token) => PeerConnectionPendingAction { address, token }.into(),
                Err(error) => PeerConnectionErrorAction {
                    address,
                    error: error.kind().into(),
                }
                .into(),
            });
        }
        Action::P2pPeerEvent(event) => {
            // when we receive first writable event from mio,
            // that's when we know that we successfuly connected
            // to the peer.
            if !event.is_writable() {
                return;
            }

            let mio_peer = match store.service().mio().get_peer(event.token()) {
                Some(peer) => peer,
                None => return,
            };
            let address = mio_peer.address;
            let peer = match store.state().peers.get(&address) {
                Some(peer) => peer,
                None => return,
            };

            match peer.status {
                PeerStatus::Connecting(PeerConnecting::Pending { .. }) => {
                    store.dispatch(PeerConnectionSuccessAction { address }.into());
                }
                _ => {}
            }
        }
        Action::PeerConnectionSuccess(action) => {
            let address = action.address;
            store.dispatch(
                PeerHandshakingInitAction {
                    address: action.address,
                }
                .into(),
            )
        }
        _ => {}
    }
}
