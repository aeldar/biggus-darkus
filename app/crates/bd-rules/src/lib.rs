//! Config plus game state to device commands (pure logic, no I/O).
//!
//! `Config` is parsed from versioned TOML; `plan` turns the current `GameState` into the
//! commands that change the board from what it already shows.

// Tests may panic on failure; that is how they fail.
#![cfg_attr(test, allow(clippy::expect_used))]

mod config;
mod plan;
mod template;

pub use config::{
    Action, ActionSpec, AnimSpec, Condition, Config, ConfigError, EventKind, FourShow, Rule,
    RuleError, RuleSpec, SixShow, Trigger,
};
pub use plan::{Output, plan};
pub use template::{Field, Template};
