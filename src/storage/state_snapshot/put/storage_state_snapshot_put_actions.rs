use serde::{Deserialize, Serialize};
use std::sync::Arc;

use storage::BlockHeaderWithHash;

use crate::request::RequestId;
use crate::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageStateSnapshotPutAction {
    pub state: Arc<State>,
}
