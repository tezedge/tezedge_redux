use redux_rs::Store;
use std::io::Write;

use crate::action::Action;
use crate::peer::PeerStatus;
use crate::service::{MioService, Service};
use crate::State;

use super::handshaking::connection_message::write::{
    PeerConnectionMessagePartWrittenAction, PeerConnectionMessageWriteErrorAction,
};
use super::handshaking::{MessageWriteState, PeerHandshakingStatus};
use super::PeerTryWriteAction;

pub fn peer_effects<S>(store: &mut Store<State, S, Action>, action: &Action)
where
    S: Service,
{
    match action {
        Action::P2pPeerEvent(event) => {
            if event.is_closed() {
                return;
            }

            if event.is_writable() {
                store.dispatch(
                    PeerTryWriteAction {
                        address: event.address(),
                    }
                    .into(),
                );
            }
        }
        Action::PeerTryWrite(action) => {
            let peer = match store.state.get().peers.get(&action.address) {
                Some(v) => v,
                None => return,
            };

            let peer_token = match &peer.status {
                PeerStatus::Handshaking(s) => s.token,
                _ => return,
            };

            let peer_stream = match store.service.mio().get_peer(peer_token) {
                Some(peer) => &mut peer.stream,
                None => return,
            };

            match &peer.status {
                PeerStatus::Handshaking(handshaking) => match &handshaking.status {
                    PeerHandshakingStatus::ConnectionMessageWrite { conn_msg, status } => {
                        let bytes = match status {
                            MessageWriteState::Idle => conn_msg.raw(),
                            MessageWriteState::Pending { written } => &conn_msg.raw()[*written..],
                            _ => return,
                        };
                        match peer_stream.write(conn_msg.raw()) {
                            Ok(written) => {
                                if written == 0 {
                                    return;
                                }
                                store.dispatch(
                                    PeerConnectionMessagePartWrittenAction {
                                        address: action.address,
                                        bytes_written: written,
                                    }
                                    .into(),
                                );
                            }
                            Err(err) => {
                                match err.kind() {
                                    std::io::ErrorKind::WouldBlock => return,
                                    _ => {}
                                }
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
                    _ => return,
                },
                _ => return,
            }
        }
        _ => {}
    }
}
