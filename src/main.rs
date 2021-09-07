use std::io;
use std::net::SocketAddrV4;
use std::time::{Duration, SystemTime};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use crypto::crypto_box::PublicKey;
use derive_more::From;
use redux_rs::Store;

use shell_proposer::mio_manager::{MioEvent, MioEvents, MioManager, NetPeer};
use shell_proposer::{Event, NetworkEvent, Events, Manager};
use shell_state::networking::peer::PeerCrypto;
use tezos_messages::p2p::{
    binary_message::BinaryChunk,
    encoding::{metadata::MetadataMessage, version::NetworkVersion},
};

mod dns;

pub type Port = u16;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum RequestState {
    // Idle { at: SystemTime },
    // Pending { at: SystemTime },
    // Success { at: SystemTime },
    // Error { at: SystemTime },
    Idle,
    Pending,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct ReceivedConnectionMessageData {
    port: Port,
    compatible_version: Option<NetworkVersion>,
    public_key: PublicKey,
    encoded: BinaryChunk,
}

#[derive(Debug, Clone)]
pub enum PeerHandshakingStatus {
    /// Exchange Connection message.
    ExchangeConnectionMessage {
        sent: RequestState,
        received: Option<ReceivedConnectionMessageData>,
        sent_conn_msg: BinaryChunk,
    },
    /// Exchange Metadata message.
    ExchangeMetadataMessage {
        sent: RequestState,
        received: Option<MetadataMessage>,

        port: Port,
        compatible_version: Option<NetworkVersion>,
        public_key: PublicKey,
        crypto: PeerCrypto,
    },
    /// Exchange Ack message.
    ExchangeAckMessage {
        sent: RequestState,
        received: bool,

        port: Port,
        compatible_version: Option<NetworkVersion>,
        public_key: PublicKey,
        disable_mempool: bool,
        private_node: bool,
        crypto: PeerCrypto,
    },
}

#[derive(Debug, Clone)]
pub enum PeerConnecting {
    Idle,
    Pending,
    Success,
    Error { error: io::ErrorKind },
}

#[derive(Debug, Clone)]
pub struct PeerHandshaking {
    pub status: PeerHandshakingStatus,
    pub incoming: bool,
}

#[derive(Debug, Clone)]
pub struct PeerHandshaked {
    pub address: SocketAddr,
    pub port: Port,
    pub version: NetworkVersion,
    pub public_key: PublicKey,
    pub crypto: PeerCrypto,
    pub disable_mempool: bool,
    pub private_node: bool,
}

#[derive(Debug, Clone)]
pub enum PeerStatus {
    /// Peer is a potential peer.
    Potential,

    Connecting(PeerConnecting),
    Handshaking(PeerHandshaking),
    Handshaked(PeerHandshaked),
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub status: PeerStatus,
}

#[derive(Debug, Clone)]
pub enum PeersDnsLookupStatus {
    Init,
    Success { addresses: Vec<SocketAddr> },
    Error { error: dns_lookup::LookupErrorKind },
}

#[derive(Debug, Clone)]
pub struct PeersDnsLookupState {
    address: String,
    port: Port,
    status: PeersDnsLookupStatus,
}

#[derive(Debug, Clone)]
struct State {
    peers: BTreeMap<SocketAddr, Peer>,
    peers_dns_lookup: Option<PeersDnsLookupState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            peers: BTreeMap::new(),
            peers_dns_lookup: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeersDnsLookupInitAction {
    pub address: String,
    pub port: Port,
}

#[derive(Debug, Clone)]
pub struct PeersDnsLookupSuccessAction {
    addresses: Vec<SocketAddr>,
}

#[derive(Debug, Clone)]
pub struct PeersDnsLookupErrorAction {
    error: dns_lookup::LookupErrorKind,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionInitAction {
    address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionPendingAction {
    address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionErrorAction {
    address: SocketAddr,
    error: io::ErrorKind,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionSuccessAction {
    address: SocketAddr,
}

#[derive(From, Debug, Clone)]
enum Action {
    PeersDnsLookupInit(PeersDnsLookupInitAction),
    PeersDnsLookupError(PeersDnsLookupErrorAction),
    PeersDnsLookupSuccess(PeersDnsLookupSuccessAction),
    PeersDnsLookupFinish,

    PeerConnectionInit(PeerConnectionInitAction),
    PeerConnectionPending(PeerConnectionPendingAction),
    PeerConnectionError(PeerConnectionErrorAction),
    PeerConnectionSuccess(PeerConnectionSuccessAction),
}

// Here comes the reducer. It gets the current state plus an action to perform and returns a new state.
fn reducer(state: &State, action: &Action) -> State {
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
        Action::PeersDnsLookupFinish => {
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

        Action::PeerConnectionInit(action) => {
            let peer = state.peers.entry(action.address).or_insert_with(|| Peer {
                status: PeerStatus::Potential,
            });
            if matches!(peer.status, PeerStatus::Potential) {
                peer.status = PeerStatus::Connecting(PeerConnecting::Idle);
            }
        }
        Action::PeerConnectionPending(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                if matches!(peer.status, PeerStatus::Connecting(PeerConnecting::Idle)) {
                    peer.status = PeerStatus::Connecting(PeerConnecting::Pending);
                }
            }
        }
        Action::PeerConnectionError(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                if matches!(
                    peer.status,
                    PeerStatus::Connecting(PeerConnecting::Idle)
                        | PeerStatus::Connecting(PeerConnecting::Pending)
                ) {
                    peer.status = PeerStatus::Connecting(PeerConnecting::Error {
                        error: action.error,
                    });
                }
            }
        }
        Action::PeerConnectionSuccess(action) => {
            if let Some(peer) = state.peers.get_mut(&action.address) {
                if matches!(peer.status, PeerStatus::Connecting(PeerConnecting::Pending)) {
                    peer.status = PeerStatus::Connecting(PeerConnecting::Success);
                }
            }
            let peer = state.peers.entry(action.address).or_insert_with(|| Peer {
                status: PeerStatus::Potential,
            });
            if matches!(peer.status, PeerStatus::Potential) {
                peer.status = PeerStatus::Connecting(PeerConnecting::Idle);
            }
        }
        _ => {}
    }
    state
}

fn effects_middleware(store: &mut Store<State, Service, Action>, action: &Action) {
    match action {
        Action::PeersDnsLookupInit(action) => {
            store.dispatch(
                match dns::resolve_dns_name_to_peer_address(&action.address, action.port) {
                    Ok(addresses) => PeersDnsLookupSuccessAction { addresses }.into(),
                    Err(err) => PeersDnsLookupErrorAction { error: err.kind() }.into(),
                },
            );
        }
        Action::PeersDnsLookupSuccess(_) => {
            store.dispatch(Action::PeersDnsLookupFinish);
        }
        Action::PeersDnsLookupFinish => {
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

fn log_middleware(store: &mut Store<State, Service, Action>, action: &Action) {
    eprintln!("[+] Action: {:#?}", &action);
    // eprintln!("[+] State: {:?}\n", store.state());
}

pub struct Service {
    mio: MioManager,
}

impl Service {
    pub fn new() -> Self {
        Self {
            mio: MioManager::new(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(0, 0, 0, 0),
                9734,
            ))),
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
    fn get_peer_for_event_mut(
        &mut self,
        event: &MioEvent,
    ) -> Option<&mut NetPeer>
    {
        self.mio.get_peer_for_event_mut(event)
    }
}

fn main() {
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
                            store.dispatch(PeerConnectionSuccessAction {
                                address,
                            }.into());
                        }
                    }
                }
            }
        }
    }
}
