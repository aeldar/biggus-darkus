//! One command at a time over a `Transport`.

use std::{io, time::Duration};

use bd_protocol::{Command, EncodeError, ErrCode, Mode, Reply};
use thiserror::Error;

use crate::Transport;

/// How many times `connect` asks before giving up; with a 1 s reply timeout that is about 8 s of boot time.
const CONNECT_ATTEMPTS: u32 = 8;

/// Why a command did not get a usable reply.
#[derive(Debug, Error)]
pub enum Error {
    /// The command does not fit on a line.
    #[error(transparent)]
    Encode(#[from] EncodeError),
    /// No reply in time.
    #[error("no reply from the board")]
    Timeout,
    /// The port failed, usually because it was unplugged.
    #[error("serial port error: {0}")]
    Io(io::Error),
    /// The board sent something that is not a reply.
    #[error("unreadable reply: {0:?}")]
    BadReply(String),
    /// The board answered `ERR` where `OK` was needed.
    #[error("board refused with ERR {}", .0.word())]
    Rejected(ErrCode),
    /// No board is connected right now.
    #[error("not connected")]
    NotConnected,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::TimedOut => Self::Timeout,
            _ => Self::Io(e),
        }
    }
}

/// A board on some transport. Every call takes `&mut self`, so only one command is in flight.
pub struct Device<T> {
    transport: T,
    timeout: Duration,
}

impl<T: Transport> Device<T> {
    /// Wrap a transport without talking to the board.
    pub fn new(transport: T, timeout: Duration) -> Self {
        Self { transport, timeout }
    }

    /// Wrap a transport and switch the board to `ACTIVE`, the only mode that takes commands.
    ///
    /// A Nano that just reset can need several seconds before its sketch reads lines, so a silent
    /// board is asked again; `MODE ACTIVE` is harmless to repeat. Other failures are final.
    pub fn connect(transport: T, timeout: Duration) -> Result<Self, Error> {
        let mut device = Self::new(transport, timeout);
        let mut attempt = 1;
        loop {
            match device.send_ok(&Command::Mode(Mode::Active)) {
                Err(Error::Timeout) if attempt < CONNECT_ATTEMPTS => attempt += 1,
                result => return result.map(|()| device),
            }
        }
    }

    /// Hand the board back to its demo mode, so it is not left blank-and-silent.
    pub fn shutdown(&mut self) -> Result<(), Error> {
        self.send_ok(&Command::Mode(Mode::Demo))
    }

    /// Send a command and wait for its reply. An `ERR` reply is a value, not an error.
    pub fn send(&mut self, command: &Command) -> Result<Reply, Error> {
        let line = command.encode()?;
        self.transport.discard_input()?;
        self.transport.write_line(line.as_str())?;
        let text = self.transport.read_line(self.timeout)?;
        text.parse().map_err(|_| Error::BadReply(text))
    }

    fn send_ok(&mut self, command: &Command) -> Result<(), Error> {
        match self.send(command)? {
            Reply::Ok => Ok(()),
            Reply::Err(code) => Err(Error::Rejected(code)),
        }
    }
}
