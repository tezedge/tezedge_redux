use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, StatusCode,
};
use redux_rs::{ActionId, ActionWithId};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::{convert::Infallible, future::Future};
use storage::{PersistentStorage, ReduxActionStorage, ReduxStateStorage, StorageError};

use crate::{action::Action, request::RequestId, State};

use super::service_channel::{
    worker_channel, RequestSendError, ResponseTryRecvError, ServiceWorkerRequester,
    ServiceWorkerResponder, ServiceWorkerResponderSender,
};

pub trait RpcService {
    /// Try to receive/read queued message, if there is any.
    fn try_recv(&mut self) -> Result<RpcResponse, ResponseTryRecvError>;
}

#[derive(Debug)]
pub enum RpcResponse {
    GetCurrentGlobalState {
        channel: tokio::sync::oneshot::Sender<State>,
    },
}

type ServiceResult = Result<Response<Body>, Box<dyn std::error::Error + Sync + Send>>;

#[derive(Debug)]
pub struct RpcServiceDefault {
    worker_channel: ServiceWorkerRequester<(), RpcResponse>,
}

#[derive(Serialize, Deserialize)]
struct ActionWithState {
    #[serde(flatten)]
    action: ActionWithId<Action>,
    state: State,
}

/// Generate 404 response
fn not_found() -> ServiceResult {
    Ok(Response::builder()
        .status(StatusCode::from_u16(404)?)
        .header(hyper::header::CONTENT_TYPE, "text/plain")
        .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type")
        .header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, "content-type")
        .body(Body::empty())?)
}

/// Function to generate JSON response from serializable object
fn make_json_response<T: serde::Serialize>(content: &T) -> ServiceResult {
    Ok(Response::builder()
        .header(hyper::header::CONTENT_TYPE, "application/json")
        // TODO: add to config
        .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type")
        .header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, "content-type")
        .header(
            hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, POST, OPTIONS, PUT",
        )
        .body(Body::from(serde_json::to_string(content)?))?)
}

impl RpcServiceDefault {
    async fn get_global_state(
        mut sender: ServiceWorkerResponderSender<RpcResponse>,
    ) -> Result<State, tokio::sync::oneshot::error::RecvError> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        sender.send(RpcResponse::GetCurrentGlobalState { channel: tx });
        rx.await
    }

    async fn handle_global_state_get(
        mut sender: ServiceWorkerResponderSender<RpcResponse>,
    ) -> ServiceResult {
        let state = Self::get_global_state(sender).await.unwrap();

        make_json_response(&state)
    }

    async fn get_action(
        action_storage: &ReduxActionStorage,
        action_id: u64,
    ) -> Result<Option<ActionWithId<Action>>, StorageError> {
        let action_storage = action_storage.clone();
        tokio::task::spawn_blocking(move || {
            Ok(action_storage
                .get::<Action>(&action_id)?
                .map(|action| ActionWithId {
                    id: ActionId::new_unchecked(action_id),
                    action,
                }))
        })
        .await
        .unwrap()
    }

    async fn handle_actions_get(
        mut sender: ServiceWorkerResponderSender<RpcResponse>,
        snapshot_storage: &ReduxStateStorage,
        action_storage: &ReduxActionStorage,
        cursor: Option<u64>,
        limit: Option<u64>,
    ) -> ServiceResult {
        // TODO: optimize by getting just part of state, instead of whole state.
        let limit = limit.unwrap_or(20);
        let cursor = match cursor {
            Some(v) => v,
            None => {
                let state = Self::get_global_state(sender).await.unwrap();
                let last_action_id_num: u64 = state.last_action_id.into();
                last_action_id_num.checked_sub(limit).unwrap_or(0)
            }
        };

        let closest_snapshot_action_id = cursor / 10000;
        let snapshot = match snapshot_storage.get(&closest_snapshot_action_id) {
            Ok(Some(v)) => v,
            Ok(None) => return make_json_response::<Vec<()>>(&vec![]),
            Err(err) => {
                dbg!(err);
                return make_json_response::<Vec<()>>(&vec![]);
            }
        };

        let mut state = snapshot;

        if cursor > closest_snapshot_action_id {
            for action_id in (closest_snapshot_action_id + 1)..cursor {
                let action = match Self::get_action(action_storage, action_id).await.unwrap() {
                    Some(v) => v,
                    None => break,
                };
                crate::reducer(&mut state, &action);
            }
        }

        let mut actions_with_state = vec![];

        let cursor = match cursor {
            // Actions start from 1.
            0 => 1,
            v => v,
        };

        for action_id in cursor..(cursor + limit) {
            let action = match Self::get_action(action_storage, action_id).await.unwrap() {
                Some(v) => v,
                None => break,
            };
            crate::reducer(&mut state, &action);
            actions_with_state.push(ActionWithState {
                action,
                state: state.clone(),
            });
        }

        make_json_response(&actions_with_state)
    }

    fn run_worker(
        bind_address: SocketAddr,
        mut channel: ServiceWorkerResponder<(), RpcResponse>,
        storage: PersistentStorage,
    ) -> impl Future<Output = Result<(), hyper::Error>> {
        let sender = channel.sender();

        let snapshot_storage = ReduxStateStorage::new(&storage);
        let action_storage = ReduxActionStorage::new(&storage);

        hyper::Server::bind(&bind_address).serve(make_service_fn(move |_| {
            let sender = sender.clone();
            let snapshot_storage = snapshot_storage.clone();
            let action_storage = action_storage.clone();

            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    let sender = sender.clone();
                    let snapshot_storage = snapshot_storage.clone();
                    let action_storage = action_storage.clone();
                    async move {
                        let path = req.uri().path();
                        if path == "/state" {
                            Self::handle_global_state_get(sender).await
                        } else if path.starts_with("/actions") {
                            let (cursor, limit) = req
                                .uri()
                                .query()
                                .map(|query| {
                                    query
                                        .split("&")
                                        .map(|x| x.trim())
                                        .filter(|x| !x.is_empty())
                                        .map(|x| {
                                            let mut parts = x.split("=");
                                            if let (Some(key), Some(value)) =
                                                (parts.next(), parts.next())
                                            {
                                                Some((key, value))
                                            } else {
                                                None
                                            }
                                        })
                                        .filter_map(|x| x)
                                        .fold((None, None), |r, (k, v)| match k {
                                            "cursor" => (v.parse().ok(), r.1),
                                            "limit" => (r.0, v.parse().ok()),
                                            _ => r,
                                        })
                                })
                                .unwrap_or_else(|| (None, None));

                            Self::handle_actions_get(
                                sender,
                                &snapshot_storage,
                                &action_storage,
                                cursor,
                                limit,
                            )
                            .await
                        } else {
                            not_found()
                        }
                    }
                }))
            }
        }))
    }

    // TODO: remove unwraps
    pub fn init(waker: Arc<mio::Waker>, storage: PersistentStorage) -> Self {
        let (requester, responder) = worker_channel(waker);

        thread::spawn(move || {
            let rpc_listen_address = ([0, 0, 0, 0], 18732).into();
            let threaded_rt = tokio::runtime::Runtime::new().unwrap();
            threaded_rt.block_on(async move {
                Self::run_worker(rpc_listen_address, responder, storage)
                    .await
                    .unwrap();
            });
        });

        Self {
            worker_channel: requester,
        }
    }
}

impl RpcService for RpcServiceDefault {
    #[inline(always)]
    fn try_recv(&mut self) -> Result<RpcResponse, ResponseTryRecvError> {
        self.worker_channel.try_recv()
    }
}
