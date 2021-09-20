use redux_rs::Store;

use crate::service::rpc_service::RpcResponse;
use crate::service::{RpcService, Service};
use crate::{action::Action, State};

pub fn rpc_effects<S: Service>(store: &mut Store<State, S, Action>, action: &Action) {
    match action {
        Action::WakeupEvent(_) => {
            while let Ok(msg) = store.service().rpc().try_recv() {
                match msg {
                    RpcResponse::GetCurrentGlobalState { channel } => {
                        channel.send(store.state.get().clone());
                    }
                }
            }
        }
        _ => {}
    }
}
