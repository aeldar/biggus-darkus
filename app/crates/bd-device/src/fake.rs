//! A Rust model of the firmware's mode and `ERR` rules, usable as a `Transport` in tests.
//! Rules come from the firmware's `docs/commands.md` and `serial_commands.cpp`.

use std::{
    collections::VecDeque,
    io,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use bd_protocol::{
    Anim, Command, ErrCode, FourDigit, LED_COUNT, LINE_MAX, Lcd, Mode, ParseError, Reply, Rgb,
    SixDigit,
};

use crate::Transport;

/// What one LED is set to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Led {
    /// Colour.
    pub color: Rgb,
    /// Animation.
    pub anim: Anim,
}

/// Everything the fake remembers about the board.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    /// Current global mode.
    pub mode: Mode,
    /// LCD text per row; empty means cleared.
    pub lcd: [String; 2],
    /// Per-LED state; `None` is off. Kept in every mode, like the firmware.
    pub leds: [Option<Led>; LED_COUNT as usize],
    /// Last 4-digit command.
    pub four_digit: Option<FourDigit>,
    /// Last 6-digit command.
    pub six_digit: Option<SixDigit>,
}

impl Board {
    /// The board at power-on: demo mode, everything blank.
    pub fn new() -> Self {
        Self {
            mode: Mode::Demo,
            lcd: Default::default(),
            leds: [None; LED_COUNT as usize],
            four_digit: None,
            six_digit: None,
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

/// The `ERR` code the firmware gives for each parse failure.
fn err_code(e: &ParseError) -> ErrCode {
    match e {
        ParseError::Line => ErrCode::Line,
        ParseError::Unknown => ErrCode::Unknown,
        ParseError::Args => ErrCode::Args,
        ParseError::Range(_) => ErrCode::Range,
    }
}

/// How the firmware answers one line, and the board after it. Pure, so tests can call it directly.
pub fn respond(board: Board, line: &str) -> (Board, Reply) {
    if line.chars().count() > LINE_MAX {
        return (board, Reply::Err(ErrCode::Line));
    }
    // The mode check runs before argument parsing, so even a malformed line gets ERR MODE.
    let is_mode = line
        .split_whitespace()
        .next()
        .is_some_and(|device| device.eq_ignore_ascii_case("MODE"));
    if !is_mode && board.mode != Mode::Active {
        return (board, Reply::Err(ErrCode::Mode));
    }
    match line.parse::<Command>() {
        Ok(command) => (apply(board, command), Reply::Ok),
        Err(e) => (board, Reply::Err(err_code(&e))),
    }
}

fn apply(mut board: Board, command: Command) -> Board {
    match command {
        Command::Mode(mode) => board.mode = mode,
        Command::FourDigit(c) => board.four_digit = Some(c),
        Command::SixDigit(c) => board.six_digit = Some(c),
        Command::Lcd(Lcd::Clear) => board.lcd = Default::default(),
        Command::Lcd(Lcd::Row(row, text)) => {
            board.lcd[usize::from(row.number() - 1)] =
                text.map(|t| t.as_str().to_owned()).unwrap_or_default();
        }
        Command::LedStrip(bd_protocol::LedStrip::Set { range, color, anim }) => {
            board.leds[usize::from(range.start())..usize::from(range.end())]
                .fill(Some(Led { color, anim }));
        }
        Command::LedStrip(bd_protocol::LedStrip::ClearRange(range)) => {
            board.leds[usize::from(range.start())..usize::from(range.end())].fill(None);
        }
        Command::LedStrip(bd_protocol::LedStrip::Clear) => board.leds.fill(None),
    }
    board
}

#[derive(Debug)]
struct Inner {
    board: Board,
    /// Reply lines the host has not read yet.
    replies: VecDeque<String>,
    /// Swallow commands without answering, to test timeouts.
    silent: bool,
    /// Every operation fails, like a pulled cable.
    unplugged: bool,
}

/// A fake board. Clones share one board, so a test keeps a handle to inspect it
/// while a `Device` owns another.
#[derive(Debug, Clone)]
pub struct FakeDevice(Arc<Mutex<Inner>>);

impl Default for FakeDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeDevice {
    /// A board in its power-on state.
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner {
            board: Board::new(),
            replies: VecDeque::new(),
            silent: false,
            unplugged: false,
        })))
    }

    // A panic in another test thread must not hide this test's result.
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Copy of the board for assertions.
    pub fn board(&self) -> Board {
        self.lock().board.clone()
    }

    /// Stop answering (or start again).
    pub fn set_silent(&self, silent: bool) {
        self.lock().silent = silent;
    }

    /// Pull the cable out (`true`) or put it back (`false`).
    pub fn set_unplugged(&self, unplugged: bool) {
        self.lock().unplugged = unplugged;
    }
}

fn gone() -> io::Error {
    io::ErrorKind::BrokenPipe.into()
}

impl Transport for FakeDevice {
    fn read_line(&mut self, _timeout: Duration) -> io::Result<String> {
        let mut inner = self.lock();
        if inner.unplugged {
            return Err(gone());
        }
        // Nothing queued means nothing will arrive, so time out at once instead of sleeping.
        inner
            .replies
            .pop_front()
            .ok_or_else(|| io::ErrorKind::TimedOut.into())
    }

    fn write_line(&mut self, line: &str) -> io::Result<()> {
        let mut inner = self.lock();
        if inner.unplugged {
            return Err(gone());
        }
        if !inner.silent {
            let (board, reply) = respond(inner.board.clone(), line);
            inner.board = board;
            inner.replies.push_back(reply.to_string());
        }
        Ok(())
    }

    fn discard_input(&mut self) -> io::Result<()> {
        let mut inner = self.lock();
        if inner.unplugged {
            return Err(gone());
        }
        inner.replies.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply_in(mode: Mode, line: &str) -> String {
        let board = Board {
            mode,
            ..Board::new()
        };
        respond(board, line).1.to_string()
    }

    #[test]
    fn error_codes() {
        let cases = [
            (Mode::Active, "FOO 1", "ERR UNKNOWN"),
            (Mode::Active, "LCD ROW 3 hi", "ERR ARGS"),
            (Mode::Active, "6DIGIT SET 1000000", "ERR RANGE"),
            (Mode::Active, "LED_STRIP RANGE 0 47 CLEAR", "ERR RANGE"),
            (Mode::Active, "6DIGIT SET", "ERR ARGS"),
            (Mode::Active, "MODE SLEEP", "ERR ARGS"),
            (Mode::Active, &"X".repeat(65), "ERR LINE"),
            (Mode::Demo, &"X".repeat(65), "ERR LINE"),
            (Mode::Demo, "LCD CLEAR", "ERR MODE"),
            (Mode::Off, "FOO 1", "ERR MODE"),
            (Mode::Off, "6DIGIT SET", "ERR MODE"),
            (Mode::Off, "MODE DEMO", "OK"),
            (Mode::Demo, "mode active", "OK"),
        ];
        for (mode, line, want) in cases {
            assert_eq!(reply_in(mode, line), want, "{mode:?} {line}");
        }
    }
}
