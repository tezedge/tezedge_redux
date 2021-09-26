use derive_more::From;
use serde::{Deserialize, Serialize};
use storage::persistent::{BincodeEncoded, SchemaError};

use crate::event::{P2pPeerEvent, WakeupEvent};
use crate::peer::connecting::{
    PeerConnectionErrorAction, PeerConnectionInitAction, PeerConnectionPendingAction,
    PeerConnectionSuccessAction,
};
use crate::peer::disconnection::{PeerDisconnectAction, PeerDisconnectedAction};
use crate::peer::handshaking::connection_message::read::{
    PeerConnectionMessagePartReadAction, PeerConnectionMessageReadErrorAction,
    PeerConnectionMessageReadInitAction, PeerConnectionMessageReadSuccessAction,
};
use crate::peer::handshaking::connection_message::write::{
    PeerConnectionMessagePartWrittenAction, PeerConnectionMessageWriteErrorAction,
    PeerConnectionMessageWriteInitAction, PeerConnectionMessageWriteSuccessAction,
};
use crate::peer::handshaking::PeerHandshakingInitAction;
use crate::peer::{PeerTryReadAction, PeerTryWriteAction};
use crate::peers::dns_lookup::{
    PeersDnsLookupErrorAction, PeersDnsLookupFinishAction, PeersDnsLookupInitAction,
    PeersDnsLookupSuccessAction,
};
use crate::peers::remove::PeersRemoveAction;
use crate::storage::block_header::put::{
    StorageBlockHeaderPutNextInitAction, StorageBlockHeaderPutNextPendingAction,
    StorageBlockHeadersPutAction,
};
use crate::storage::request::{
    StorageRequestCreateAction, StorageRequestErrorAction, StorageRequestFinishAction,
    StorageRequestInitAction, StorageRequestPendingAction, StorageRequestSuccessAction,
};
use crate::storage::state_snapshot::create::StorageStateSnapshotCreateAction;

#[derive(From, Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "content")]
pub enum Action {
    PeersDnsLookupInit(PeersDnsLookupInitAction),
    PeersDnsLookupError(PeersDnsLookupErrorAction),
    PeersDnsLookupSuccess(PeersDnsLookupSuccessAction),
    PeersDnsLookupFinish(PeersDnsLookupFinishAction),

    PeersRemove(PeersRemoveAction),

    PeerConnectionInit(PeerConnectionInitAction),
    PeerConnectionPending(PeerConnectionPendingAction),
    PeerConnectionError(PeerConnectionErrorAction),
    PeerConnectionSuccess(PeerConnectionSuccessAction),

    PeerDisconnect(PeerDisconnectAction),
    PeerDisconnected(PeerDisconnectedAction),

    P2pPeerEvent(P2pPeerEvent),
    WakeupEvent(WakeupEvent),

    PeerTryWrite(PeerTryWriteAction),
    PeerTryRead(PeerTryReadAction),

    PeerHandshakingInit(PeerHandshakingInitAction),

    PeerConnectionMessageWriteInit(PeerConnectionMessageWriteInitAction),
    PeerConnectionMessagePartWritten(PeerConnectionMessagePartWrittenAction),
    PeerConnectionMessageWriteError(PeerConnectionMessageWriteErrorAction),
    PeerConnectionMessageWriteSuccess(PeerConnectionMessageWriteSuccessAction),

    PeerConnectionMessageReadInit(PeerConnectionMessageReadInitAction),
    PeerConnectionMessagePartRead(PeerConnectionMessagePartReadAction),
    PeerConnectionMessageReadError(PeerConnectionMessageReadErrorAction),
    PeerConnectionMessageReadSuccess(PeerConnectionMessageReadSuccessAction),

    StorageBlockHeadersPut(StorageBlockHeadersPutAction),
    StorageBlockHeaderPutNextInit(StorageBlockHeaderPutNextInitAction),
    StorageBlockHeaderPutNextPending(StorageBlockHeaderPutNextPendingAction),

    StorageStateSnapshotCreate(StorageStateSnapshotCreateAction),

    StorageRequestCreate(StorageRequestCreateAction),
    StorageRequestInit(StorageRequestInitAction),
    StorageRequestPending(StorageRequestPendingAction),
    StorageRequestError(StorageRequestErrorAction),
    StorageRequestSuccess(StorageRequestSuccessAction),
    StorageRequestFinish(StorageRequestFinishAction),
}

// bincode decoding fails with: "Bincode does not support Deserializer::deserialize_identifier".
// So use json instead, which works.

// impl BincodeEncoded for Action {
//     fn decode(bytes: &[u8]) -> Result<Self, storage::persistent::SchemaError> {
//         // here it errors.
//         Ok(dbg!(bincode::deserialize(bytes)).unwrap())
//     }

//     fn encode(&self) -> Result<Vec<u8>, storage::persistent::SchemaError> {
//         Ok(bincode::serialize::<Self>(self).unwrap())
//     }
// }

impl storage::persistent::Encoder for Action {
    fn encode(&self) -> Result<Vec<u8>, SchemaError> {
        serde_json::to_vec(self).map_err(|_| SchemaError::EncodeError)
    }
}

impl storage::persistent::Decoder for Action {
    fn decode(bytes: &[u8]) -> Result<Self, SchemaError> {
        serde_json::from_slice(bytes).map_err(|_| SchemaError::DecodeError)
    }
}
