use crate::{State, action::Action, peer::{Peer, PeerStatus}};

use super::{PeersDnsLookupState, PeersDnsLookupStatus};

pub fn peers_dns_lookup_reducer(state: &State, action: &Action) -> State {
    let mut state = state.clone();
    match action {
        Action::PeersDnsLookupInit(action) => {
            state.peers_dns_lookup = Some(PeersDnsLookupState {
                address: action.address.clone(),
                port: action.port,
                status: PeersDnsLookupStatus::Init,
            });
        }
        Action::PeersDnsLookupError(action) => {
            if let Some(dns_lookup_state) = state.peers_dns_lookup.as_mut() {
                if let PeersDnsLookupStatus::Init = dns_lookup_state.status {
                    dns_lookup_state.status = PeersDnsLookupStatus::Error {
                        error: action.error,
                    };
                }
            }
        }
        Action::PeersDnsLookupSuccess(action) => {
            if let Some(dns_lookup_state) = state.peers_dns_lookup.as_mut() {
                if let PeersDnsLookupStatus::Init = dns_lookup_state.status {
                    dns_lookup_state.status = PeersDnsLookupStatus::Success {
                        addresses: action.addresses.clone(),
                    };
                }
            }
        }
        Action::PeersDnsLookupFinish(_) => {
            if let Some(dns_lookup_state) = state.peers_dns_lookup.take() {
                if let PeersDnsLookupStatus::Success { addresses } = dns_lookup_state.status {
                    for address in addresses {
                        state.peers.entry(address).or_insert_with(|| Peer {
                            status: PeerStatus::Potential,
                        });
                    }
                }
            };
        }
        _ => {}
    }
    state
}
