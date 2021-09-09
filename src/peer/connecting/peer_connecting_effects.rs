use redux_rs::Store;

use crate::event::Event;
use crate::mio_service::MioService;
use crate::peer::PeerStatus;
use crate::{action::Action, Service, State};

use super::{
    PeerConnecting, PeerConnectionErrorAction, PeerConnectionPendingAction,
    PeerConnectionSuccessAction,
};

pub fn peer_connecting_effects<Mio>(store: &mut Store<State, Service<Mio>, Action>, action: &Action)
where
    Mio: MioService,
{
    match action {
        Action::PeerConnectionInit(action) => {
            let address = action.address;
            let result = store.service().mio().peer_connection_init(address);
            store.dispatch(match result {
                Ok(token) => PeerConnectionPendingAction { address, token }.into(),
                Err(error) => PeerConnectionErrorAction {
                    address,
                    error: error.kind(),
                }
                .into(),
            });
        }
        Action::Event(event) => {
            let event = match event {
                Event::Network(event) => event,
                _ => return,
            };
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
        _ => {}
    }
}
