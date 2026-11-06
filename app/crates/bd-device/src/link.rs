//! Keeping a device connected: pure state changes plus a small driver around them.

use std::{io, time::Duration};

use bd_protocol::{Command, Reply};

use crate::{Device, Error, Transport};

/// Where the connection stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Never connected, or stopped on purpose.
    Disconnected,
    /// Board is answering.
    Connected,
    /// The board went away; this many reconnect attempts have failed so far.
    Reconnecting {
        /// Failed attempts since the loss.
        attempt: u32,
    },
}

/// Things that change the connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkEvent {
    /// Opened the port and the board accepted `MODE ACTIVE`.
    Connected,
    /// An attempt to connect failed.
    ConnectFailed,
    /// The port failed while connected.
    Lost,
}

/// The state machine, kept pure so it can be tested as a table.
pub fn transition(state: ConnectionState, event: LinkEvent) -> ConnectionState {
    use ConnectionState::*;
    match (state, event) {
        (_, LinkEvent::Connected) => Connected,
        (Connected, LinkEvent::Lost) => Reconnecting { attempt: 0 },
        (Reconnecting { attempt }, LinkEvent::ConnectFailed) => Reconnecting {
            attempt: attempt + 1,
        },
        (state, _) => state,
    }
}

/// How long to wait before the next reconnect attempt: 0.5 s, doubling, capped at 8 s.
pub fn retry_delay(attempt: u32) -> Duration {
    Duration::from_millis(500 << attempt.min(4))
}

/// A device that can be reopened. `open` makes a fresh transport each time.
pub struct Link<T, F> {
    open: F,
    timeout: Duration,
    device: Option<Device<T>>,
    state: ConnectionState,
    /// Why the last connect attempt failed, for the user to read.
    last_error: Option<String>,
}

impl<T: Transport, F: FnMut() -> io::Result<T>> Link<T, F> {
    /// Start disconnected; nothing is opened until `try_connect`.
    pub fn new(open: F, timeout: Duration) -> Self {
        Self {
            open,
            timeout,
            device: None,
            state: ConnectionState::Disconnected,
            last_error: None,
        }
    }

    /// Current state.
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Why the last connect attempt failed, if it did.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// One attempt to open the port and activate the board. The caller sleeps `retry_delay` between attempts.
    pub fn try_connect(&mut self) -> ConnectionState {
        let timeout = self.timeout;
        let device = (self.open)()
            .map_err(Error::from)
            .and_then(|t| Device::connect(t, timeout));
        let event = match device {
            Ok(device) => {
                self.device = Some(device);
                self.last_error = None;
                LinkEvent::Connected
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                LinkEvent::ConnectFailed
            }
        };
        self.state = transition(self.state, event);
        self.state
    }

    /// Send through the current device; a port failure drops it and starts reconnecting.
    pub fn send(&mut self, command: &Command) -> Result<Reply, Error> {
        let device = self.device.as_mut().ok_or(Error::NotConnected)?;
        let result = device.send(command);
        if matches!(result, Err(Error::Io(_))) {
            self.device = None;
            self.state = transition(self.state, LinkEvent::Lost);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ConnectionState::*;
    use LinkEvent as E;

    #[test]
    fn transitions() {
        let table = [
            (Disconnected, E::Connected, Connected),
            (Disconnected, E::ConnectFailed, Disconnected),
            (Disconnected, E::Lost, Disconnected),
            (Connected, E::Lost, Reconnecting { attempt: 0 }),
            (Connected, E::ConnectFailed, Connected),
            (
                Reconnecting { attempt: 0 },
                E::ConnectFailed,
                Reconnecting { attempt: 1 },
            ),
            (Reconnecting { attempt: 3 }, E::Connected, Connected),
            (
                Reconnecting { attempt: 3 },
                E::Lost,
                Reconnecting { attempt: 3 },
            ),
        ];
        for (from, event, to) in table {
            assert_eq!(transition(from, event), to, "{from:?} + {event:?}");
        }
    }

    #[test]
    fn delay_doubles_then_caps() {
        let ms: Vec<u128> = [0, 1, 2, 3, 4, 5, 100]
            .map(|a| retry_delay(a).as_millis())
            .into();
        assert_eq!(ms, [500, 1000, 2000, 4000, 8000, 8000, 8000]);
    }
}
