use redux_rs::{ActionWithId, Store};
use tezos_messages::p2p::binary_message::BinaryChunk;

use crate::{
    action::Action,
    peer::{
        binary_message::write::peer_binary_message_write_state::PeerBinaryMessageWriteState,
        PeerStatus, PeerTryWriteAction,
    },
    service::Service,
    State,
};

use super::{
    peer_chunk_write_state::{PeerChunkWrite, PeerChunkWriteError, PeerChunkWriteState},
    PeerChunkWriteCreateChunkAction, PeerChunkWriteEncryptContentAction, PeerChunkWriteErrorAction,
    PeerChunkWriteReadyAction,
};

pub fn peer_chunk_write_effects<S>(
    store: &mut Store<State, S, Action>,
    action: &ActionWithId<Action>,
) where
    S: Service,
{
    match &action.action {
        Action::PeerChunkWriteSetContent(action) => {
            if let Some(peer) = store.state.get().peers.get(&action.address) {
                match &peer.status {
                    PeerStatus::Handshaking(handshaking) => match &handshaking.status {
                        crate::peer::handshaking::PeerHandshakingStatus::ConnectionMessageWritePending { chunk_state: PeerChunkWriteState::UnencryptedContent { content }, .. } => {
                            match BinaryChunk::from_content(&content) {
                                Ok(chunk) => store.dispatch(PeerChunkWriteCreateChunkAction { address: action.address, chunk }.into()),
                                Err(err) => store.dispatch(PeerChunkWriteErrorAction { address: action.address, error: err.into() }.into()),
                            }
                        },
                        crate::peer::handshaking::PeerHandshakingStatus::MetadataMessageWritePending { binary_message_state, .. } |
                        crate::peer::handshaking::PeerHandshakingStatus::AckMessageWritePending { binary_message_state, .. } => match binary_message_state {
                            PeerBinaryMessageWriteState::Pending { chunk: PeerChunkWrite { crypto,  state: PeerChunkWriteState::UnencryptedContent { content },  }, .. } => {
                                match crypto.encrypt(&content) {
                                    Ok(encrypted_content) => store.dispatch(PeerChunkWriteEncryptContentAction { address: action.address, encrypted_content }.into()),
                                    Err(err) => store.dispatch(PeerChunkWriteErrorAction { address: action.address, error: PeerChunkWriteError::from(err) }.into()),
                                };
                            },
                            _ => return,
                        },
                        _ => return,
                    }
                    _ => return,
                };
            }
        }
        Action::PeerChunkWriteEncryptContent(action) => {
            if let Some(peer) = store.state.get().peers.get(&action.address) {
                match &peer.status {
                    PeerStatus::Handshaking(handshaking) => match &handshaking.status {
                        crate::peer::handshaking::PeerHandshakingStatus::MetadataMessageWritePending { binary_message_state, .. } |
                        crate::peer::handshaking::PeerHandshakingStatus::AckMessageWritePending { binary_message_state, .. } => match binary_message_state {
                            PeerBinaryMessageWriteState::Pending { chunk: PeerChunkWrite { state: PeerChunkWriteState::EncryptedContent { content }, .. }, .. } =>
                                match BinaryChunk::from_content(&content) {
                                    Ok(chunk) => store.dispatch(PeerChunkWriteCreateChunkAction { address: action.address, chunk }.into()),
                                    Err(err) => store.dispatch(PeerChunkWriteErrorAction { address: action.address, error: err.into() }.into()),
                                }
                            _ => {},
                        }
                        _ => {},
                    }
                    _ => {},
                };
            }
        }
        Action::PeerChunkWriteCreateChunk(action) => {
            if let Some(peer) = store.state.get().peers.get(&action.address) {
                match &peer.status {
                    PeerStatus::Handshaking(handshaking) => match &handshaking.status {
                        crate::peer::handshaking::PeerHandshakingStatus::ConnectionMessageWritePending { chunk_state: PeerChunkWriteState::Pending { .. }, .. } => {
                            store.dispatch(PeerTryWriteAction { address: action.address }.into());
                        }
                        crate::peer::handshaking::PeerHandshakingStatus::MetadataMessageWritePending { binary_message_state, .. } |
                        crate::peer::handshaking::PeerHandshakingStatus::AckMessageWritePending { binary_message_state, .. } => match binary_message_state {
                            PeerBinaryMessageWriteState::Pending { chunk: PeerChunkWrite { state: PeerChunkWriteState::Pending { .. }, ..}, .. } => {
                                store.dispatch(PeerTryWriteAction { address: action.address }.into());
                            }
                            _ => {}
                        },
                        _ => return,
                    }
                    _ => return,
                }
            }
        }
        Action::PeerChunkWritePart(action) => {
            if let Some(peer) = store.state.get().peers.get(&action.address) {
                match &peer.status {
                    PeerStatus::Handshaking(handshaking) => match &handshaking.status {
                        crate::peer::handshaking::PeerHandshakingStatus::ConnectionMessageWritePending { chunk_state, .. } => match chunk_state {
                            PeerChunkWriteState::Pending { .. } => {
                                store.dispatch(PeerTryWriteAction { address: action.address }.into());
                            }
                            PeerChunkWriteState::Ready { .. } => {
                                store.dispatch(PeerChunkWriteReadyAction { address: action.address }.into());
                            }
                            _ => {}
                        },
                        crate::peer::handshaking::PeerHandshakingStatus::MetadataMessageWritePending { binary_message_state, .. } |
                        crate::peer::handshaking::PeerHandshakingStatus::AckMessageWritePending { binary_message_state, .. } => match binary_message_state {
                            PeerBinaryMessageWriteState::Pending { chunk, .. } => match &chunk.state {
                                PeerChunkWriteState::Pending { .. } => {
                                    store.dispatch(PeerTryWriteAction { address: action.address }.into());
                                }
                                PeerChunkWriteState::Ready { .. } => {
                                    store.dispatch(PeerChunkWriteReadyAction { address: action.address }.into());
                                }
                                _ => {}
                            }
                            _ => {}
                        },
                        _ => return,
                    }
                    _ => return,
                };
            }
        }
        _ => {}
    }
}
