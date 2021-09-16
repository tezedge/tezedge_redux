use storage::BlockHeaderWithHash;

use crate::request::RequestId;

#[derive(Debug, Clone)]
pub struct StorageBlockHeadersPutAction {
    pub block_headers: Vec<BlockHeaderWithHash>,
}

#[derive(Debug, Clone)]
pub struct StorageBlockHeaderPutNextInitAction;

#[derive(Debug, Clone)]
pub struct StorageBlockHeaderPutNextPendingAction {
    pub req_id: RequestId,
}
