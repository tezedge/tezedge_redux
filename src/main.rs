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
use peers::dns_lookup::{
    peers_dns_lookup_effects, peers_dns_lookup_reducer, DefaultPeersDnsLookupService,
    PeersDnsLookupInitAction, PeersDnsLookupState,
};
use redux_rs::{combine_reducers, Reducer, Store};

pub mod peer;
pub mod peers;

pub mod action;

pub mod event;
use event::{Event, Events};

pub mod mio_service;
use mio_service::{MioEvents, MioService, MioServiceDefault};

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

fn effects_middleware<Mio: MioService>(
    store: &mut Store<State, Service<Mio>, Action>,
    action: &Action,
) {
    peer_connecting_effects(store, action);
    peers_dns_lookup_effects(store, action);
}

fn log_middleware<Mio>(store: &mut Store<State, Service<Mio>, Action>, action: &Action) {
    eprintln!("[+] Action: {:#?}", &action);
    // eprintln!("[+] State: {:?}\n", store.state());
}

pub struct Service<Mio> {
    // TODO: use generics with traits instead
    dns: DefaultPeersDnsLookupService,
    mio: Mio,
}

impl<Mio> Service<Mio>
where
    Mio: MioService,
{
    pub fn new(mio: Mio) -> Self {
        Self {
            dns: DefaultPeersDnsLookupService {},
            mio,
        }
    }

    pub fn dns(&mut self) -> &mut DefaultPeersDnsLookupService {
        &mut self.dns
    }

    pub fn mio(&mut self) -> &mut Mio {
        &mut self.mio
    }
}

fn main() {
    let mio = MioServiceDefault::new(SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        9734,
    )));

    let reducer: Reducer<State, Action> = combine_reducers!(
        State,
        Action,
        peers_dns_lookup_reducer,
        peer_connecting_reducer
    );

    let service = Service::new(mio);
    let mut store = Store::new(reducer, service, State::new());

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
        store.service().mio().wait_for_events(&mut events, None);
        for event in events.into_iter() {
            store.dispatch(Action::Event(event.to_owned()));
        }
    }
}
