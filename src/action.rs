use derive_more::From;

use crate::peer::connecting::{
    PeerConnectionErrorAction, PeerConnectionInitAction, PeerConnectionPendingAction,
    PeerConnectionSuccessAction,
};
use crate::peers::dns_lookup::{
    PeersDnsLookupErrorAction, PeersDnsLookupFinishAction, PeersDnsLookupInitAction,
    PeersDnsLookupSuccessAction,
};

#[derive(From, Debug, Clone)]
pub enum Action {
    PeersDnsLookupInit(PeersDnsLookupInitAction),
    PeersDnsLookupError(PeersDnsLookupErrorAction),
    PeersDnsLookupSuccess(PeersDnsLookupSuccessAction),
    PeersDnsLookupFinish(PeersDnsLookupFinishAction),

    PeerConnectionInit(PeerConnectionInitAction),
    PeerConnectionPending(PeerConnectionPendingAction),
    PeerConnectionError(PeerConnectionErrorAction),
    PeerConnectionSuccess(PeerConnectionSuccessAction),
}
