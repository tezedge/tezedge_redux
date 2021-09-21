use bytes::Buf;
use redux_rs::{Store, ActionWithId};
use std::io::{Read, Write};
use tezos_messages::p2p::binary_message::CONTENT_LENGTH_FIELD_BYTES;

use super::{
    StorageBlockHeaderPutNextInitAction, StorageBlockHeaderPutNextPendingAction,
    StorageBlockHeaderPutState, StorageBlockHeadersPutAction, StorageRequestInitAction,
};
use crate::action::Action;
use crate::service::{MioService, Service};
use crate::State;

pub fn storage_block_headers_put_effects<S>(store: &mut Store<State, S, Action>, action: &ActionWithId<Action>)
where
    S: Service,
{
    match &action.action {
        Action::StorageBlockHeadersPut(_) => {
            store.dispatch(StorageBlockHeaderPutNextInitAction {}.into());
        }
        Action::StorageBlockHeaderPutNextInit(_) => {
            match store.state.get().storage.block_headers_put.front() {
                Some(StorageBlockHeaderPutState::Init { req_id, .. }) => {
                    store.dispatch(
                        StorageBlockHeaderPutNextPendingAction {
                            req_id: req_id.clone(),
                        }
                        .into(),
                    );
                }
                _ => {}
            }
        }
        Action::StorageBlockHeaderPutNextPending(action) => {
            store.dispatch(
                StorageRequestInitAction {
                    req_id: action.req_id.clone(),
                }
                .into(),
            );
            store.dispatch(StorageBlockHeaderPutNextInitAction {}.into());
        }
        _ => {}
    }
}
