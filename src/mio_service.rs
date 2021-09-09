// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use mio::net::{TcpListener, TcpSocket, TcpStream};
use slab::Slab;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::event::{Event, EventRef, Events, NetworkEvent};

/// We will receive listening socket server events with this token, when
/// there are incoming connections that need to be accepted.
pub const MIO_SERVER_TOKEN: mio::Token = mio::Token(usize::MAX);

/// Event with this token will be issued, when `mio::Waker::wake` is called.
pub const MIO_WAKE_TOKEN: mio::Token = mio::Token(usize::MAX - 1);

pub type MioEvent = mio::event::Event;
pub type EventDefault = mio::event::Event;
pub type MioPeerDefault = MioPeer<TcpStream>;

pub trait MioService {
    type PeerStream;
    type NetworkEvent;
    type Events;

    fn wait_for_events(&mut self, events: &mut Self::Events, timeout: Option<Duration>);

    fn start_listening_to_incoming_peer_connections(&mut self) -> io::Result<()>;
    fn stop_listening_to_incoming_peer_connections(&mut self);
    fn accept_incoming_peer_connection(
        &mut self,
        event: &Self::NetworkEvent,
    ) -> Option<(mio::Token, &mut MioPeer<Self::PeerStream>)>;

    fn peer_connection_init(&mut self, address: SocketAddr) -> io::Result<mio::Token>;
    fn peer_disconnect(&mut self, token: mio::Token);

    fn get_peer(&mut self, token: mio::Token) -> Option<&mut MioPeer<Self::PeerStream>>;
}

pub struct MioPeer<Stream> {
    pub address: SocketAddr,
    pub stream: Stream,
}

impl<Stream> MioPeer<Stream> {
    pub fn new(address: SocketAddr, stream: Stream) -> Self {
        Self { address, stream }
    }
}

impl NetworkEvent for EventDefault {
    #[inline(always)]
    fn token(&self) -> mio::Token {
        MioEvent::token(self)
    }

    #[inline(always)]
    fn is_server_event(&self) -> bool {
        self.token() == MIO_SERVER_TOKEN
    }

    #[inline(always)]
    fn is_waker_event(&self) -> bool {
        self.token() == MIO_WAKE_TOKEN
    }

    #[inline(always)]
    fn is_readable(&self) -> bool {
        MioEvent::is_readable(self)
    }

    #[inline(always)]
    fn is_writable(&self) -> bool {
        MioEvent::is_writable(self)
    }

    #[inline(always)]
    fn is_closed(&self) -> bool {
        MioEvent::is_error(self)
            || MioEvent::is_read_closed(self)
            || MioEvent::is_write_closed(self)
    }
}

/// Mio events container.
#[derive(Debug)]
pub struct MioEvents {
    mio_events: mio::Events,
    tick_event_time: Option<Instant>,
}

impl MioEvents {
    pub fn new() -> Self {
        Self {
            mio_events: mio::Events::with_capacity(0),
            tick_event_time: None,
        }
    }
}

impl<'a> IntoIterator for &'a MioEvents {
    type Item = EventRef<'a, MioEvent>;
    type IntoIter = MioEventsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let tick_event_time = self
            .tick_event_time
            .filter(|_| self.mio_events.is_empty())
            .clone();
        MioEventsIter {
            mio_events_iter: self.mio_events.iter(),
            tick_event_time,
        }
    }
}

pub struct MioEventsIter<'a> {
    mio_events_iter: mio::event::Iter<'a>,
    tick_event_time: Option<Instant>,
}

impl<'a> Iterator for MioEventsIter<'a> {
    type Item = EventRef<'a, MioEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let tick_event_time = &mut self.tick_event_time;

        self.mio_events_iter
            .next()
            .map(|event| EventRef::Network(event))
            .or_else(|| tick_event_time.take().map(|time| Event::Tick(time)))
    }
}

impl Events for MioEvents {
    fn set_limit(&mut self, limit: usize) {
        self.mio_events = mio::Events::with_capacity(limit);
    }
}

pub struct MioServiceDefault {
    listen_addr: SocketAddr,

    /// Backlog size for incoming connections.
    ///
    /// Incoming connections are put in kernel's backlog, this is limit
    /// for that backlog. So if queue of incoming connections get to
    /// this limit, more connections will be instantly rejected.
    backlog_size: u32,

    poll: mio::Poll,
    waker: Arc<mio::Waker>,
    server: Option<TcpListener>,

    peers: Slab<MioPeer<TcpStream>>,
}

impl MioServiceDefault {
    const DEFAULT_BACKLOG_SIZE: u32 = 255;

    pub fn new(listen_addr: SocketAddr) -> Self {
        let poll = mio::Poll::new().expect("failed to create mio::Poll");
        let waker = Arc::new(
            mio::Waker::new(poll.registry(), MIO_WAKE_TOKEN).expect("failed to create mio::Waker"),
        );
        Self {
            listen_addr,
            backlog_size: Self::DEFAULT_BACKLOG_SIZE,
            poll,
            waker,
            server: None,
            peers: Slab::new(),
        }
    }

    /// Waker can be used to wake up mio from another thread.
    pub fn waker(&self) -> Arc<mio::Waker> {
        self.waker.clone()
    }
}

impl MioService for MioServiceDefault {
    type PeerStream = TcpStream;
    type NetworkEvent = MioEvent;
    type Events = MioEvents;

    fn wait_for_events(&mut self, events: &mut Self::Events, timeout: Option<Duration>) {
        match self.poll.poll(&mut events.mio_events, timeout) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Mio Poll::poll failed! Error: {:?}", err);
            }
        };

        if events.mio_events.is_empty() {
            events.tick_event_time = Some(Instant::now());
        }
    }

    fn start_listening_to_incoming_peer_connections(&mut self) -> io::Result<()> {
        if self.server.is_none() {
            let socket = match self.listen_addr.ip() {
                IpAddr::V4(_) => TcpSocket::new_v4()?,
                IpAddr::V6(_) => TcpSocket::new_v6()?,
            };

            // read more details about why not on windows in mio docs
            // for [mio::TcpListener::bind].
            #[cfg(not(windows))]
            socket.set_reuseaddr(true)?;

            socket.bind(self.listen_addr)?;

            let mut server = socket.listen(self.backlog_size)?;

            self.poll.registry().register(
                &mut server,
                MIO_SERVER_TOKEN,
                mio::Interest::READABLE,
            )?;

            self.server = Some(server);
        }
        Ok(())
    }

    fn stop_listening_to_incoming_peer_connections(&mut self) {
        drop(self.server.take());
    }

    fn accept_incoming_peer_connection(
        &mut self,
        _: &Self::NetworkEvent,
    ) -> Option<(mio::Token, &mut MioPeer<Self::PeerStream>)> {
        let server = &mut self.server;
        let poll = &mut self.poll;
        let peers = &mut self.peers;

        if let Some(server) = server.as_mut() {
            match server.accept() {
                Ok((mut stream, address)) => {
                    let peer_entry = peers.vacant_entry();
                    let token = mio::Token(peer_entry.key());

                    let registered_poll = poll.registry().register(
                        &mut stream,
                        token,
                        mio::Interest::READABLE | mio::Interest::WRITABLE,
                    );

                    match registered_poll {
                        Ok(_) => {
                            let peer = peer_entry.insert(MioPeer::new(address.into(), stream));
                            Some((token, peer))
                        }
                        Err(err) => {
                            eprintln!("error while registering poll: {:?}", err);
                            None
                        }
                    }
                }
                Err(err) => {
                    match err.kind() {
                        io::ErrorKind::WouldBlock => {}
                        _ => {
                            eprintln!("error while accepting connection: {:?}", err);
                        }
                    }
                    None
                }
            }
        } else {
            None
        }
    }

    fn peer_connection_init(&mut self, address: SocketAddr) -> io::Result<mio::Token> {
        let poll = &mut self.poll;
        let peers = &mut self.peers;

        let peer_entry = peers.vacant_entry();
        let token = mio::Token(peer_entry.key());

        match TcpStream::connect(address) {
            Ok(mut stream) => {
                poll.registry().register(
                    &mut stream,
                    token,
                    mio::Interest::READABLE | mio::Interest::WRITABLE,
                )?;

                let peer = MioPeer::new(address.clone(), stream);

                peer_entry.insert(peer);
                Ok(token)
            }
            Err(err) => Err(err),
        }
    }

    fn peer_disconnect(&mut self, token: mio::Token) {
        let index = token.into();
        if self.peers.contains(index) {
            let mut peer = self.peers.remove(index);
            let _ = self.poll.registry().deregister(&mut peer.stream);
        }
    }

    fn get_peer(&mut self, token: mio::Token) -> Option<&mut MioPeer<Self::PeerStream>> {
        self.peers.get_mut(token.into())
    }
}
