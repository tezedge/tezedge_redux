use redux_rs::{ActionWithId, Store};

use crate::peer::handshaking::PeerHandshakingInitAction;
use crate::peer::PeerStatus;
use crate::service::{MioService, RandomnessService, Service};
use crate::{action::Action, State};

use super::{
    PeerConnectionOutgoingErrorAction, PeerConnectionOutgoingInitAction,
    PeerConnectionOutgoingPendingAction, PeerConnectionOutgoingRandomInitAction,
    PeerConnectionOutgoingState, PeerConnectionOutgoingSuccessAction,
};

pub fn peer_connection_outgoing_effects<S>(
    store: &mut Store<State, S, Action>,
    action: &ActionWithId<Action>,
) where
    S: Service,
{
    match &action.action {
        Action::PeerConnectionOutgoingRandomInit(_) => {
            let addresses = store
                .state
                .get()
                .peers
                .iter()
                .filter(|(_, peer)| matches!(&peer.status, PeerStatus::Potential))
                .map(|(addr, _)| *addr)
                .collect::<Vec<_>>();

            if let Some(address) = store.service.randomness().choose_peer(&addresses) {
                store.dispatch(PeerConnectionOutgoingInitAction { address }.into());
            }
        }
        Action::PeerConnectionOutgoingInit(action) => {
            let address = action.address;
            let result = store.service().mio().peer_connection_init(address);
            store.dispatch(match result {
                Ok(token) => PeerConnectionOutgoingPendingAction { address, token }.into(),
                Err(error) => PeerConnectionOutgoingErrorAction {
                    address,
                    error: error.kind().into(),
                }
                .into(),
            });
        }
        Action::PeerConnectionOutgoingPending(_) => {
            // try to connect to next random peer.
            // TODO: maybe check peer thresholds?
            store.dispatch(PeerConnectionOutgoingRandomInitAction {}.into());
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
                PeerStatus::Connecting(PeerConnectionOutgoingState::Pending { .. }) => {
                    store.dispatch(PeerConnectionOutgoingSuccessAction { address }.into());
                }
                _ => {}
            }
        }
        Action::PeerConnectionOutgoingSuccess(action) => {
            let address = action.address;
            store.dispatch(
                PeerHandshakingInitAction {
                    address: action.address,
                }
                .into(),
            )
        }
        Action::PeerConnectionOutgoingError(_) => {
            store.dispatch(PeerConnectionOutgoingRandomInitAction {}.into());
        }
        _ => {}
    }
}
