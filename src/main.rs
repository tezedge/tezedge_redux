use redux_rs::Store;

pub mod io_error_kind;

pub mod event;
use event::Event;

pub mod action;

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
use persistent_storage::{gen_block_headers, init_storage};

pub type Port = u16;

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
