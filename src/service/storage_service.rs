use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::thread;
use redux_rs::ActionWithId;

use storage::{BlockHeaderWithHash, BlockStorage, PersistentStorage, StorageError};

use crate::action::Action;
use crate::request::RequestId;

use super::service_channel::{
    worker_channel, RequestSendError, ResponseTryRecvError, ServiceWorkerRequester,
    ServiceWorkerResponder,
};

pub trait StorageService {
    /// Send the request to storage for execution.
    fn request_send(&mut self, req: StorageRequest)
        -> Result<(), RequestSendError<StorageRequest>>;

    /// Try to receive/read queued response, if there is any.
    fn response_try_recv(&mut self) -> Result<StorageResponse, ResponseTryRecvError>;

    fn action_store(&mut self, action: &ActionWithId<Action>);

    fn actions_get(&self) -> Vec<ActionWithId<Action>>;
}

type StorageWorkerRequester = ServiceWorkerRequester<StorageRequest, StorageResponse>;
type StorageWorkerResponder = ServiceWorkerResponder<StorageRequest, StorageResponse>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageErrorTmp;

impl From<StorageError> for StorageErrorTmp {
    fn from(_: StorageError) -> Self {
        Self {}
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageRequestPayload {
    BlockHeaderWithHashPut(BlockHeaderWithHash),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageResponseSuccess {
    BlockHeaderWithHashPutSuccess(bool),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageResponseError {
    BlockHeaderWithHashPutError(StorageErrorTmp),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageRequest {
    pub id: RequestId,
    pub payload: StorageRequestPayload,
}

impl StorageRequest {
    pub fn new(id: RequestId, payload: StorageRequestPayload) -> Self {
        Self { id, payload }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageResponse {
    pub req_id: RequestId,
    pub result: Result<StorageResponseSuccess, StorageResponseError>,
}

impl StorageResponse {
    pub fn new(
        req_id: RequestId,
        result: Result<StorageResponseSuccess, StorageResponseError>,
    ) -> Self {
        Self { req_id, result }
    }
}

#[derive(Debug)]
pub struct StorageServiceDefault {
    worker_channel: StorageWorkerRequester,
    actions: Vec<ActionWithId<Action>>,
}

impl StorageServiceDefault {
    fn run_worker(storage: PersistentStorage, mut channel: StorageWorkerResponder) {
        use StorageRequestPayload::*;
        use StorageResponseError::*;
        use StorageResponseSuccess::*;

        let block_storage = BlockStorage::new(&storage);

        while let Ok(req) = channel.recv() {
            match req.payload {
                BlockHeaderWithHashPut(block_header_with_hash) => {
                    let result = block_storage
                        .put_block_header(&block_header_with_hash)
                        .map(|res| BlockHeaderWithHashPutSuccess(res))
                        .map_err(|err| BlockHeaderWithHashPutError(err.into()));

                    let _ = channel.send(StorageResponse::new(req.id, result));
                }
            }
        }
    }

    // TODO: remove unwraps
    pub fn init(waker: Arc<mio::Waker>, persistent_storage: PersistentStorage) -> Self {
        let (requester, responder) = worker_channel(waker);

        thread::Builder::new()
            .name("storage-thread".to_owned())
            .spawn(move || Self::run_worker(persistent_storage, responder))
            .unwrap();

        Self {
            worker_channel: requester,
            actions: vec![],
        }
    }
}

impl StorageService for StorageServiceDefault {
    #[inline(always)]
    fn request_send(
        &mut self,
        req: StorageRequest,
    ) -> Result<(), RequestSendError<StorageRequest>> {
        self.worker_channel.send(req)
    }

    #[inline(always)]
    fn response_try_recv(&mut self) -> Result<StorageResponse, ResponseTryRecvError> {
        self.worker_channel.try_recv()
    }

    fn action_store(&mut self, action: &ActionWithId<Action>) {
        self.actions.push(action.clone());
    }

    fn actions_get(&self) -> Vec<ActionWithId<Action>> {
        self.actions.clone()
    }
}
