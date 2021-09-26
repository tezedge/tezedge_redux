use std::collections::BTreeMap;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use redux_rs::ActionId;

use ::storage::persistent::BincodeEncoded;

use crate::storage::StorageState;
use crate::config::Config;
use crate::peers::dns_lookup::PeersDnsLookupState;
use crate::peer::Peer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub config: Config,
    pub peers: BTreeMap<SocketAddr, Peer>,
    pub peers_dns_lookup: Option<PeersDnsLookupState>,
    pub storage: StorageState,
    pub last_action_id: ActionId,
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            peers: BTreeMap::new(),
            peers_dns_lookup: None,
            storage: StorageState::new(),
            last_action_id: ActionId::ZERO,
        }
    }
}

impl BincodeEncoded for State {}
