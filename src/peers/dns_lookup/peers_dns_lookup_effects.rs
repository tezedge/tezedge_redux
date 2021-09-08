use redux_rs::Store;

use crate::peer::PeerStatus;
use crate::peer::connecting::PeerConnectionInitAction;
use crate::{action::Action, Service, State};
use crate::peers::dns_lookup::PeersDnsLookupService;

use super::{PeersDnsLookupErrorAction, PeersDnsLookupFinishAction, PeersDnsLookupSuccessAction};

pub fn peers_dns_lookup_effects(store: &mut Store<State, Service, Action>, action: &Action) {
    match action {
        Action::PeersDnsLookupInit(action) => {
            let result = store.service().dns.resolve_dns_name_to_peer_address(&action.address, action.port);
            store.dispatch(
                match result {
                    Ok(addresses) => PeersDnsLookupSuccessAction { addresses }.into(),
                    Err(err) => PeersDnsLookupErrorAction { error: err.kind() }.into(),
                },
            );
        }
        Action::PeersDnsLookupSuccess(_) => {
            store.dispatch(PeersDnsLookupFinishAction.into());
        }
        Action::PeersDnsLookupFinish(_) => {
            // Try connecting to first potential peer we find.
            let address = store
                .state()
                .peers
                .iter()
                .filter(|(_, peer)| matches!(peer.status, PeerStatus::Potential))
                .map(|(address, _)| address)
                .next();

            if let Some(&address) = address {
                store.dispatch(PeerConnectionInitAction { address }.into());
            }
        }
        _ => {}
    }
}
