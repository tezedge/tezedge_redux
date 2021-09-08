use redux_rs::Store;

use crate::{action::Action, Service, State};

use super::{PeerConnectionErrorAction, PeerConnectionPendingAction};

pub fn peer_connecting_effects(store: &mut Store<State, Service, Action>, action: &Action) {
    match action {
        Action::PeerConnectionInit(action) => {
            let address = action.address.clone();
            let result = store.service().peer_connection_init(&address);
            store.dispatch(match result {
                Ok(_) => PeerConnectionPendingAction { address }.into(),
                Err(error) => PeerConnectionErrorAction {
                    address,
                    error: error.kind(),
                }
                .into(),
            });
        }
        _ => {}
    }
}
