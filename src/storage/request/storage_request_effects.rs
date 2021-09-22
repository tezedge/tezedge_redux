use bytes::Buf;
use redux_rs::{ActionWithId, Store};
use std::io::{Read, Write};
use tezos_messages::p2p::binary_message::CONTENT_LENGTH_FIELD_BYTES;

use crate::action::Action;
use crate::service::storage_service::{StorageRequest, StorageRequestPayload};
use crate::service::{MioService, Service, StorageService};
use crate::storage::block_header::put::StorageBlockHeaderPutNextInitAction;
use crate::State;

use super::{
    StorageRequestErrorAction, StorageRequestFinishAction, StorageRequestInitAction,
    StorageRequestPendingAction, StorageRequestStatus, StorageRequestSuccessAction,
};

pub fn storage_request_effects<S>(
    store: &mut Store<State, S, Action>,
    action: &ActionWithId<Action>,
) where
    S: Service,
{
    match &action.action {
        Action::StorageRequestCreate(_) => {
            store.dispatch(
                StorageRequestInitAction {
                    req_id: store.state().storage.requests.last_added_req_id(),
                }
                .into(),
            );
        }
        Action::StorageRequestInit(action) => {
            let req = match store.state.get().storage.requests.get(action.req_id) {
                Some(v) => v,
                None => return,
            };
            match req.status {
                StorageRequestStatus::Idle => {}
                _ => return,
            }
            // TODO: handle send error in case of mpsc disconnection.
            store
                .service
                .storage()
                .request_send(StorageRequest {
                    id: action.req_id,
                    payload: req.payload.clone(),
                })
                .unwrap();
            store.dispatch(
                StorageRequestPendingAction {
                    req_id: action.req_id,
                }
                .into(),
            );
        }
        Action::WakeupEvent(_) => {
            // TODO: handle disconnected error.
            while let Ok(resp) = store.service.storage().response_try_recv() {
                store.dispatch(match resp.result {
                    Ok(result) => StorageRequestSuccessAction {
                        req_id: resp.req_id,
                        result,
                    }
                    .into(),
                    Err(error) => StorageRequestErrorAction {
                        req_id: resp.req_id,
                        error,
                    }
                    .into(),
                });
            }
        }
        Action::StorageRequestError(action) => {
            store.dispatch(
                StorageRequestFinishAction {
                    req_id: action.req_id,
                }
                .into(),
            );
            store.dispatch(StorageBlockHeaderPutNextInitAction {}.into());
        }
        Action::StorageRequestSuccess(action) => {
            store.dispatch(
                StorageRequestFinishAction {
                    req_id: action.req_id,
                }
                .into(),
            );
            store.dispatch(StorageBlockHeaderPutNextInitAction {}.into());
        }
        _ => {}
    }
}
