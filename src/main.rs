use std::io;
use std::net::SocketAddrV4;
use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, SocketAddr},
};

use action::Action;
use peer::connecting::{
    peer_connecting_effects, peer_connecting_reducer, PeerConnectionSuccessAction,
};
use peer::Peer;
use peers::dns_lookup::{DefaultPeersDnsLookupService, PeersDnsLookupInitAction, PeersDnsLookupState, peers_dns_lookup_effects, peers_dns_lookup_reducer};
use redux_rs::{Reducer, Store, combine_reducers};

use shell_proposer::mio_manager::{MioEvent, MioEvents, MioManager, NetPeer};
use shell_proposer::{Event, Events, Manager, NetworkEvent};

pub mod peer;
pub mod peers;

pub mod action;

pub type Port = u16;

#[derive(Debug, Clone)]
pub struct State {
    pub peers: BTreeMap<SocketAddr, Peer>,
    pub peers_dns_lookup: Option<PeersDnsLookupState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            peers: BTreeMap::new(),
            peers_dns_lookup: None,
        }
    }
}

fn effects_middleware(store: &mut Store<State, Service, Action>, action: &Action) {
    peer_connecting_effects(store, action);
    peers_dns_lookup_effects(store, action);
}

fn log_middleware(store: &mut Store<State, Service, Action>, action: &Action) {
    eprintln!("[+] Action: {:#?}", &action);
    // eprintln!("[+] State: {:?}\n", store.state());
}

pub struct Service {
    mio: MioManager,
    // TODO: use generic instead
    pub dns: DefaultPeersDnsLookupService,
}

impl Service {
    pub fn new() -> Self {
        Self {
            mio: MioManager::new(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(0, 0, 0, 0),
                9734,
            ))),
            dns: DefaultPeersDnsLookupService {},
        }
    }

    pub fn wait_for_events(&mut self, events: &mut MioEvents) {
        self.mio.wait_for_events(events, None)
    }

    pub fn peer_connection_init(&mut self, address: &SocketAddr) -> io::Result<()> {
        // TODO: rename method on mio.
        self.mio.get_peer_or_connect_mut(&address.into())?;
        Ok(())
    }

    /// Get peer for which the event was received.
    ///
    /// mio::Event contains `mio::Token`, which is used to find the peer,
    /// but when mocking, this method could work some other way.
    fn get_peer_for_event_mut(&mut self, event: &MioEvent) -> Option<&mut NetPeer> {
        self.mio.get_peer_for_event_mut(event)
    }
}

fn main() {
    let reducer: Reducer<State, Action> = combine_reducers!(
        State,
        Action,
        peers_dns_lookup_reducer,
        peer_connecting_reducer
    );
    let mut store = Store::new(reducer, Service::new(), State::new());

    store.add_middleware(log_middleware);
    store.add_middleware(effects_middleware);

    store.dispatch(
        PeersDnsLookupInitAction {
            address: "boot.tzbeta.net".to_owned(),
            port: 9732,
        }
        .into(),
    );

    let mut events = MioEvents::new();
    events.set_limit(1024);

    loop {
        store.service().wait_for_events(&mut events);
        for event in events.into_iter() {
            match event {
                Event::Tick(_) => {}
                Event::Network(event) => {
                    let peer = match store.service().get_peer_for_event_mut(event) {
                        Some(peer) => peer,
                        None => continue,
                    };
                    // when we receive first writable event from mio,
                    // that's when we know that we successfuly connected
                    // to the peer.
                    if event.is_writable() {
                        if !peer.is_connected() {
                            peer.set_connected();
                            let address = peer.address().into();
                            store.dispatch(PeerConnectionSuccessAction { address }.into());
                        }
                    }
                }
            }
        }
    }
}
