//! Elite Dangerous journal files to typed events to game state (no serial, no GUI).
//!
//! Official journal documentation: <https://elite-journal.readthedocs.io/en/latest/>

// Tests may panic on failure; that is how they fail.
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic))]
pub mod dir;
pub mod event;
pub mod state;
pub mod status;
pub mod tail;
pub mod watch;

pub use event::Event;
pub use state::{GameState, apply, apply_status};
pub use status::Status;
pub use tail::{DirSource, ReplaySource, Source, Tailer};
