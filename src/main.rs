use std::net::SocketAddrV4;
use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, SocketAddr},
};

use action::Action;
use peer::connecting::{
    peer_connecting_effects, peer_connecting_reducer, PeerConnectionSuccessAction,
};
use peer::handshaking::connection_message::write::peer_connection_message_write_effects;
use peer::handshaking::peer_handshaking_effects;
use peer::{peer_effects, Peer};
use peers::dns_lookup::{
    peers_dns_lookup_effects, peers_dns_lookup_reducer, PeersDnsLookupInitAction,
    PeersDnsLookupState,
};
use redux_rs::{combine_reducers, Reducer, Store};

pub mod peer;
pub mod peers;

pub mod action;

pub mod event;
use event::Event;

pub mod config;
use config::{default_config, Config};

pub mod service;
use service::mio_service::MioInternalEventsContainer;
use service::{
    DnsServiceDefault, MioService, MioServiceDefault, RandomnessServiceDefault, Service,
    ServiceDefault,
};

use crate::peer::handshaking::connection_message::write::peer_connection_message_write_reducer;
use crate::peer::handshaking::peer_handshaking_reducer;

pub type Port = u16;

#[derive(Debug, Clone)]
pub struct State {
    pub config: Config,
    pub peers: BTreeMap<SocketAddr, Peer>,
    pub peers_dns_lookup: Option<PeersDnsLookupState>,
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            peers: BTreeMap::new(),
            peers_dns_lookup: None,
        }
    }
}

fn effects_middleware<S: Service>(store: &mut Store<State, S, Action>, action: &Action) {
    peers_dns_lookup_effects(store, action);
    peer_effects(store, action);
    peer_connecting_effects(store, action);
    peer_handshaking_effects(store, action);
    peer_connection_message_write_effects(store, action);
}

fn log_middleware<S: Service>(store: &mut Store<State, S, Action>, action: &Action) {
    eprintln!("[+] Action: {:#?}", &action);
    // eprintln!("[+] State: {:?}\n", store.state());
}

fn main() {
    let listen_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9734));

    let service = ServiceDefault {
        randomness: RandomnessServiceDefault::default(),
        dns: DnsServiceDefault::default(),
        mio: MioServiceDefault::new(listen_address),
    };

    let reducer: Reducer<State, Action> = combine_reducers!(
        State,
        Action,
        peers_dns_lookup_reducer,
        peer_connecting_reducer,
        peer_handshaking_reducer,
        peer_connection_message_write_reducer
    );

    let mut store = Store::new(reducer, service, State::new(default_config()));

    store.add_middleware(log_middleware);
    store.add_middleware(effects_middleware);

    store.dispatch(
        PeersDnsLookupInitAction {
            address: "boot.tzbeta.net".to_owned(),
            port: 9732,
        }
        .into(),
    );

    let mut events = MioInternalEventsContainer::with_capacity(1024);

    loop {
        store.service().mio().wait_for_events(&mut events, None);
        for event in events.into_iter() {
            match store.service().mio().transform_event(event) {
                Event::P2pPeer(p2p_peer_event) => store.dispatch(p2p_peer_event.into()),
                _ => {}
            }
        }
    }
}
