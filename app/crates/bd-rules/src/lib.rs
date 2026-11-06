//! Config plus game state to device commands (pure logic, no I/O).
//!
//! `Config` is parsed from versioned TOML; `plan` turns the current `GameState` into the
//! commands that change the board from what it already shows.

mod config;
mod plan;
mod template;

pub use config::{Action, Condition, Config, ConfigError, EventKind, Rule, RuleError, Trigger};
pub use plan::{Output, plan};
pub use template::{Field, Template};
