use redux_rs::{ActionWithId, Store};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::sync::Arc;

use ::storage::BlockHeaderWithHash;
use crypto::hash::BlockHash;
use tezos_messages::p2p::binary_message::{BinaryRead, MessageHash};
use tezos_messages::p2p::encoding::block_header::{BlockHeader, BlockHeaderBuilder, Fitness};

pub mod io_error_kind;

pub mod event;
use event::{Event, WakeupEvent};

pub mod action;
use action::Action;

pub mod config;
use config::default_config;

mod state;
pub use state::State;

mod reducer;
pub use reducer::reducer;

mod effects;
pub use effects::effects;

pub mod request;

pub mod peer;

pub mod peers;
use peers::dns_lookup::PeersDnsLookupInitAction;

pub mod storage;
use crate::storage::block_header::put::StorageBlockHeadersPutAction;
use crate::storage::state_snapshot::create::StorageStateSnapshotCreateAction;

pub mod rpc;

pub mod service;
use crate::service::RpcServiceDefault;
use service::mio_service::MioInternalEventsContainer;
use service::{
    DnsServiceDefault, MioService, MioServiceDefault, RandomnessServiceDefault, Service,
    ServiceDefault, StorageServiceDefault,
};

pub mod persistent_storage;
use persistent_storage::init_storage;

pub type Port = u16;

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
    let listen_address = ([0, 0, 0, 0], 9734).into();

    let persistent_storage = init_storage();

    let mio_service = MioServiceDefault::new(listen_address);
    let storage_service =
        StorageServiceDefault::init(mio_service.waker(), persistent_storage.clone());
    let rpc_service = RpcServiceDefault::init(mio_service.waker(), persistent_storage.clone());

    let service = ServiceDefault {
        randomness: RandomnessServiceDefault::default(),
        dns: DnsServiceDefault::default(),
        mio: mio_service,
        storage: storage_service,
        rpc: rpc_service,
    };

    let mut store = Store::new(reducer, service, State::new(default_config()));

    store.add_middleware(effects);

    // Persist initial state.
    store.dispatch(StorageStateSnapshotCreateAction {}.into());

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
