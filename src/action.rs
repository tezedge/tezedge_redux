use derive_more::From;

use crate::event::P2pPeerEvent;
use crate::peer::connecting::{
    PeerConnectionErrorAction, PeerConnectionInitAction, PeerConnectionPendingAction,
    PeerConnectionSuccessAction,
};
use crate::peer::handshaking::connection_message::write::{
    PeerConnectionMessagePartWrittenAction, PeerConnectionMessageWriteErrorAction,
    PeerConnectionMessageWriteInitAction, PeerConnectionMessageWriteSuccessAction,
};
use crate::peer::handshaking::PeerHandshakingInitAction;
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

    PeerHandshakingInit(PeerHandshakingInitAction),

    PeerConnectionMessageWriteInit(PeerConnectionMessageWriteInitAction),
    PeerConnectionMessagePartWritten(PeerConnectionMessagePartWrittenAction),
    PeerConnectionMessageWriteError(PeerConnectionMessageWriteErrorAction),
    PeerConnectionMessageWriteSuccess(PeerConnectionMessageWriteSuccessAction),

    P2pPeerEvent(P2pPeerEvent),
}
