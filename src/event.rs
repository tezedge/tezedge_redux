// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::time::Instant;

/// Event coming from `Manager`.
///
/// Each event updates internal logical clock and also triggers some actions.
#[derive(Debug, Clone)]
pub enum Event<NetE> {
    /// TickEvent is just meant to update internal clock, when there are
    /// no new events coming from Manager (mio).
    Tick(Instant),
    /// Event from network event source, like p2p socket.
    Network(NetE),
}

impl<NetE> Event<NetE> {
    pub fn as_event_ref<'a>(&'a self) -> EventRef<'a, NetE> {
        match self {
            Self::Tick(e) => EventRef::Tick(*e),
            Self::Network(e) => EventRef::Network(e),
        }
    }
}

impl<NetE: NetworkEvent> Event<NetE> {
    pub fn time(&self) -> Instant {
        match self {
            Self::Tick(t) => t.clone(),
            Self::Network(e) => e.time(),
        }
    }
}

impl<NetE> From<NetE> for Event<NetE>
where
    NetE: NetworkEvent,
{
    fn from(event: NetE) -> Self {
        Self::Network(event)
    }
}

pub type EventRef<'a, NetE> = Event<&'a NetE>;

impl<'a, NetE: Clone> EventRef<'a, NetE> {
    pub fn to_owned(self) -> Event<NetE> {
        match self {
            Self::Tick(at) => Event::Tick(at),
            Self::Network(e) => Event::Network(NetE::clone(e)),
        }
    }
}

pub trait NetworkEvent {
    fn token(&self) -> mio::Token;
    /// If event source is server.
    ///
    /// This usually means that we have incoming connections that we need
    /// to "accept" using `Manager::accept_connection`.
    fn is_server_event(&self) -> bool;

    /// Event was caused by waking up the waker.
    fn is_waker_event(&self) -> bool;

    /// Source for which this event is, is ready for reading.
    fn is_readable(&self) -> bool;

    /// Source for which this event is, is ready for writing.
    fn is_writable(&self) -> bool;

    /// Is event source closed.
    fn is_closed(&self) -> bool;

    /// Time associated with event.
    ///
    /// By default it's `Instant::now()`, during testing can be overriden.
    /// Using this time, internal clock of the state machine is updated.
    fn time(&self) -> Instant {
        Instant::now()
    }
}

pub trait Events {
    /// Set limit on how many events might the container contain.
    fn set_limit(&mut self, limit: usize);
}
