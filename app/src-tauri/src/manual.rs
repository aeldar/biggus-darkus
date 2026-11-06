//! Typing a protocol command by hand: parse it first, and list what can be typed.

use bd_protocol::{Command, CommandKind, spec};
use serde::{Deserialize, Serialize};
use specta::Type;

/// One kind of command, described for the hint list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct CommandHelp {
    /// Short heading.
    pub title: String,
    /// What the command does.
    pub description: String,
    /// Shape of the line; `[x]` is optional.
    pub syntax: String,
    /// A line that is valid as it stands.
    pub example: String,
}

/// Every command kind with its example, straight from the protocol spec.
pub fn help() -> Vec<CommandHelp> {
    CommandKind::ALL
        .into_iter()
        .map(spec)
        .map(|s| CommandHelp {
            title: s.title.to_owned(),
            description: s.description.to_owned(),
            syntax: s.syntax.to_owned(),
            example: s.example.to_owned(),
        })
        .collect()
}

/// Parses what was typed. Ok carries the command; its text is the canonical upper-case line.
pub fn check(text: &str) -> Result<Command, String> {
    text.trim().parse::<Command>().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_text_becomes_a_canonical_command() {
        let c = check("  lcd row 1 Hello ").expect("valid");
        assert_eq!(c.to_string(), "LCD ROW 1 Hello");
    }

    #[test]
    fn bad_text_is_an_error_message() {
        assert_eq!(check("").expect_err("empty"), "unknown device");
        assert_eq!(
            check("LED_STRIP RANGE 5 3 COLOR FF0000 ANIM SOLID").expect_err("range"),
            "LED range out of range"
        );
        assert_eq!(
            check("MODE").expect_err("args"),
            "missing or malformed argument"
        );
        assert!(check(&"X".repeat(70)).is_err());
    }

    #[test]
    fn every_hint_example_is_valid() {
        let hints = help();
        assert_eq!(hints.len(), CommandKind::ALL.len());
        for h in hints {
            assert_eq!(check(&h.example).expect("example").to_string(), h.example);
        }
    }
}
