use ::storage::BlockHeaderWithHash;
use crypto::hash::BlockHash;
use redux_rs::{combine_reducers, Reducer, Store};
use std::convert::TryInto;
use std::net::SocketAddrV4;
use std::sync::Arc;
use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, SocketAddr},
};
use tezos_messages::p2p::binary_message::{BinaryRead, MessageHash};
use tezos_messages::p2p::encoding::block_header::{BlockHeader, BlockHeaderBuilder, Fitness};

pub mod event;
use event::Event;

pub mod action;
use action::Action;

pub mod config;
use config::{default_config, Config};

pub mod request;

pub mod peer;
use peer::connecting::{peer_connecting_effects, peer_connecting_reducer};
use peer::handshaking::connection_message::read::{
    peer_connection_message_read_effects, peer_connection_message_read_reducer,
};
use peer::handshaking::connection_message::write::{
    peer_connection_message_write_effects, peer_connection_message_write_reducer,
};
use peer::handshaking::{peer_handshaking_effects, peer_handshaking_reducer};
use peer::{peer_effects, Peer};

pub mod peers;
use peers::dns_lookup::{
    peers_dns_lookup_effects, peers_dns_lookup_reducer, PeersDnsLookupInitAction,
    PeersDnsLookupState,
};

pub mod storage;
use crate::storage::{StorageBlockHeadersPutAction, StorageState};

pub mod service;
use service::mio_service::MioInternalEventsContainer;
use service::{
    DnsServiceDefault, MioService, MioServiceDefault, RandomnessServiceDefault, Service,
    ServiceDefault, StorageServiceDefault,
};

pub mod persistent_storage;
use persistent_storage::init_storage;

use crate::event::WakeupEvent;
use crate::storage::{
    storage_block_headers_put_effects, storage_block_headers_put_reducer, storage_request_effects,
    storage_request_reducer,
};

pub type Port = u16;

#[derive(Debug, Clone)]
pub struct State {
    pub config: Config,
    pub peers: BTreeMap<SocketAddr, Peer>,
    pub peers_dns_lookup: Option<PeersDnsLookupState>,
    pub storage: StorageState,
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            peers: BTreeMap::new(),
            peers_dns_lookup: None,
            storage: StorageState::new(),
        }
    }
}

fn log_middleware<S: Service>(store: &mut Store<State, S, Action>, action: &Action) {
    eprintln!("[+] Action: {:#?}", &action);
    // eprintln!("[+] State: {:#?}\n", store.state());
}

fn gen_block_headers() -> Vec<BlockHeaderWithHash> {
    let mut builder = BlockHeaderBuilder::default();
    builder
        .level(34)
        .proto(1)
        .predecessor(
            "BKyQ9EofHrgaZKENioHyP4FZNsTmiSEcVmcghgzCC9cGhE7oCET"
                .try_into()
                .unwrap(),
        )
        .timestamp(5_635_634)
        .validation_pass(4)
        .operations_hash(
            "LLoaGLRPRx3Zf8kB4ACtgku8F4feeBiskeb41J1ciwfcXB3KzHKXc"
                .try_into()
                .unwrap(),
        )
        .fitness(Fitness::new())
        .context(
            "CoVmAcMV64uAQo8XvfLr9VDuz7HVZLT4cgK1w1qYmTjQNbGwQwDd"
                .try_into()
                .unwrap(),
        )
        .protocol_data(vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

    let header_1 = builder
        .clone()
        .level(1)
        .timestamp(5_635_634)
        .predecessor(
            "BKyQ9EofHrgaZKENioHyP4FZNsTmiSEcVmcghgzCC9cGhE7oCET"
                .try_into()
                .unwrap(),
        )
        .build()
        .unwrap();
    let header_1_hash: BlockHash = header_1.message_hash().unwrap().try_into().unwrap();

    let header_2 = builder
        .clone()
        .level(2)
        .timestamp(5_635_635)
        .predecessor(header_1_hash.clone())
        .build()
        .unwrap();
    let header_2_hash: BlockHash = header_2.message_hash().unwrap().try_into().unwrap();

    let header_3 = builder
        .clone()
        .level(3)
        .timestamp(5_635_636)
        .predecessor(header_2_hash.clone())
        .build()
        .unwrap();

    let header_3_hash: BlockHash = header_3.message_hash().unwrap().try_into().unwrap();

    vec![
        BlockHeaderWithHash {
            hash: header_1_hash,
            header: Arc::new(header_1),
        },
        BlockHeaderWithHash {
            hash: header_2_hash,
            header: Arc::new(header_2),
        },
        BlockHeaderWithHash {
            hash: header_3_hash,
            header: Arc::new(header_3),
        },
    ]
}

fn main() {
    let listen_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9734));

    let mio_service = MioServiceDefault::new(listen_address);
    let storage_service = StorageServiceDefault::init(mio_service.waker(), init_storage());

    let service = ServiceDefault {
        randomness: RandomnessServiceDefault::default(),
        dns: DnsServiceDefault::default(),
        mio: mio_service,
        storage: storage_service,
    };

    let reducer: Reducer<State, Action> = combine_reducers!(
        State,
        Action,
        peers_dns_lookup_reducer,
        peer_connecting_reducer,
        peer_handshaking_reducer,
        peer_connection_message_write_reducer,
        peer_connection_message_read_reducer,
        storage_block_headers_put_reducer,
        storage_request_reducer
    );

    let mut store = Store::new(reducer, service, State::new(default_config()));

    store.add_middleware(log_middleware);

    // add effects middleware
    store.add_middleware(|store, action| {
        peers_dns_lookup_effects(store, action);
        peer_effects(store, action);
        peer_connecting_effects(store, action);
        peer_handshaking_effects(store, action);
        peer_connection_message_write_effects(store, action);
        peer_connection_message_read_effects(store, action);

        storage_block_headers_put_effects(store, action);
        storage_request_effects(store, action);
    });

    store.dispatch(
        PeersDnsLookupInitAction {
            address: "boot.tzbeta.net".to_owned(),
            port: 9732,
        }
        .into(),
    );
    store.dispatch(
        StorageBlockHeadersPutAction {
            block_headers: gen_block_headers(),
        }
        .into(),
    );

    let mut events = MioInternalEventsContainer::with_capacity(1024);

    loop {
        store.service().mio().wait_for_events(&mut events, None);
        for event in events.into_iter() {
            match store.service().mio().transform_event(event) {
                Event::P2pPeer(p2p_peer_event) => store.dispatch(p2p_peer_event.into()),
                Event::Wakeup(wakeup_event) => store.dispatch(wakeup_event.into()),
                _ => {}
            }
        }
    }
}
