use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use storage::BlockHeaderWithHash;

use crate::request::{PendingRequests, RequestId};
use crate::service::storage_service::{
    StorageRequestPayload, StorageResponseError, StorageResponseSuccess,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageBlockHeaderPutState {
    Idle(BlockHeaderWithHash),
    Init {
        block_header: BlockHeaderWithHash,
        req_id: RequestId,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageRequestStatus {
    Idle,
    Pending,
    Error(StorageResponseError),
    Success(StorageResponseSuccess),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageRequestState {
    pub status: StorageRequestStatus,
    pub payload: StorageRequestPayload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageState {
    pub block_headers_put: VecDeque<StorageBlockHeaderPutState>,
    pub requests: PendingRequests<StorageRequestState>,
}

impl StorageState {
    pub fn new() -> Self {
        Self {
            block_headers_put: VecDeque::new(),
            requests: PendingRequests::new(),
        }
    }
}
