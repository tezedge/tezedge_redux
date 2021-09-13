use redux_rs::Store;
use std::io::Write;
use std::net::SocketAddr;

use tezos_messages::p2p::binary_message::BinaryChunk;

use crate::action::Action;
use crate::peer::handshaking::{MessageWriteState, PeerHandshakingStatus};
use crate::peer::PeerStatus;
use crate::service::{MioService, Service};
use crate::State;

use super::{
    PeerConnectionMessagePartWrittenAction, PeerConnectionMessageWriteErrorAction,
    PeerConnectionMessageWriteSuccessAction,
};

fn map_state(
    state: &State,
    peer_address: SocketAddr,
) -> Option<(mio::Token, &BinaryChunk, &MessageWriteState)> {
    let peer = match state.peers.get(&peer_address) {
        Some(peer) => peer,
        None => return None,
    };
    match &peer.status {
        PeerStatus::Handshaking(handshaking) => {
            let (conn_msg, status) = match &handshaking.status {
                PeerHandshakingStatus::ConnectionMessageWrite { conn_msg, status } => {
                    (conn_msg, status)
                }
                _ => return None,
            };
            Some((handshaking.token, conn_msg, status))
        }
        _ => return None,
    }
}

pub fn peer_connection_message_write_effects<S>(
    store: &mut Store<State, S, Action>,
    action: &Action,
) where
    S: Service,
{
    match action {
        Action::PeerConnectionMessageWriteInit(action) => {
            let (peer_token, conn_msg, status) = match map_state(store.state.get(), action.address)
            {
                Some(v) => v,
                None => return,
            };

            let peer_stream = match store.service.mio().get_peer(peer_token) {
                Some(peer) => &mut peer.stream,
                None => return,
            };

            match peer_stream.write(conn_msg.raw()) {
                Ok(written) => {
                    store.dispatch(
                        PeerConnectionMessagePartWrittenAction {
                            address: action.address,
                            bytes_written: written,
                        }
                        .into(),
                    );
                }
                Err(err) => {
                    store.dispatch(
                        PeerConnectionMessageWriteErrorAction {
                            address: action.address,
                            error: err.kind(),
                        }
                        .into(),
                    );
                }
            }
        }
        Action::PeerConnectionMessagePartWritten(action) => {
            let (peer_token, conn_msg, status) = match map_state(store.state.get(), action.address)
            {
                Some(v) => v,
                None => return,
            };

            let index = match status {
                MessageWriteState::Pending { written } => {
                    if *written == conn_msg.raw().len() {
                        store.dispatch(
                            PeerConnectionMessageWriteSuccessAction {
                                address: action.address,
                            }
                            .into(),
                        );
                        return;
                    }
                    *written
                }
                _ => return,
            };

            let peer_stream = match store.service.mio().get_peer(peer_token) {
                Some(peer) => &mut peer.stream,
                None => return,
            };

            // Message is not yet fully written, so try to write rest of it.
            match peer_stream.write(&conn_msg.raw()[index..]) {
                Ok(written) => {
                    store.dispatch(
                        PeerConnectionMessagePartWrittenAction {
                            address: action.address,
                            bytes_written: written,
                        }
                        .into(),
                    );
                }
                Err(err) => {
                    store.dispatch(
                        PeerConnectionMessageWriteErrorAction {
                            address: action.address,
                            error: err.kind(),
                        }
                        .into(),
                    );
                }
            }
        }
        _ => {}
    }
}
