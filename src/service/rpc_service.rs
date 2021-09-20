use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, StatusCode,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::{convert::Infallible, future::Future};

use crate::{request::RequestId, State};

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

#[derive(Debug)]
pub struct RpcServiceDefault {
    worker_channel: ServiceWorkerRequester<(), RpcResponse>,
}

impl RpcServiceDefault {
    async fn handle_get_global_state(
        mut sender: ServiceWorkerResponderSender<RpcResponse>,
    ) -> Result<Response<Body>, Infallible> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        sender.send(RpcResponse::GetCurrentGlobalState { channel: tx });
        let state = match rx.await {
            Ok(v) => v,
            Err(_) => {
                return Ok(Response::new(Body::from(
                    "request for current state discarded!",
                )))
            }
        };

        let json_str = match serde_json::to_string(&state) {
            Ok(v) => v,
            Err(err) => {
                return Ok(Response::builder()
                    .status(StatusCode::from_u16(500).unwrap())
                    .header(hyper::header::CONTENT_TYPE, "text/plain")
                    .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type")
                    .header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, "content-type")
                    .header(hyper::header::TRANSFER_ENCODING, "chunked")
                    .body(Body::from(format!(
                        "serializing state failed! Error: {:?}",
                        err
                    )))
                    .unwrap());
            }
        };

        Ok(Response::new(Body::from(json_str)))
    }
    fn run_worker(
        bind_address: SocketAddr,
        mut channel: ServiceWorkerResponder<(), RpcResponse>,
    ) -> impl Future<Output = Result<(), hyper::Error>> {
        let sender = channel.sender();

        hyper::Server::bind(&bind_address).serve(make_service_fn(move |_| {
            let sender = sender.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |_req: Request<Body>| {
                    Self::handle_get_global_state(sender.clone())
                }))
            }
        }))
    }

    // TODO: remove unwraps
    pub fn init(waker: Arc<mio::Waker>) -> Self {
        let (requester, responder) = worker_channel(waker);

        thread::spawn(move || {
            let rpc_listen_address = ([0, 0, 0, 0], 18732).into();
            let threaded_rt = tokio::runtime::Runtime::new().unwrap();
            threaded_rt.block_on(async move {
                Self::run_worker(rpc_listen_address, responder)
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
