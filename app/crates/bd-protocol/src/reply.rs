//! What the board answers after each line.

use std::{fmt, str::FromStr};

use crate::command::ParseError;

/// Error codes after `ERR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrCode {
    /// Device command sent outside `ACTIVE`.
    Mode,
    /// Unknown device.
    Unknown,
    /// Missing or malformed argument.
    Args,
    /// Number or range out of bounds.
    Range,
    /// Line longer than 64 characters.
    Line,
}

impl ErrCode {
    /// Every code, in the order the docs list them.
    pub const ALL: [Self; 5] = [
        Self::Mode,
        Self::Unknown,
        Self::Args,
        Self::Range,
        Self::Line,
    ];

    /// The word after `ERR`.
    pub fn word(self) -> &'static str {
        match self {
            Self::Mode => "MODE",
            Self::Unknown => "UNKNOWN",
            Self::Args => "ARGS",
            Self::Range => "RANGE",
            Self::Line => "LINE",
        }
    }

    /// One-line meaning for the generated docs.
    pub fn meaning(self) -> &'static str {
        match self {
            Self::Mode => "Device command sent outside `ACTIVE`",
            Self::Unknown => "Unknown device",
            Self::Args => "Missing or malformed argument",
            Self::Range => "Number or range out of bounds",
            Self::Line => "Line longer than 64 characters",
        }
    }
}

/// One reply line from the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reply {
    /// Command applied.
    Ok,
    /// Command refused.
    Err(ErrCode),
}

impl fmt::Display for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => f.write_str("OK"),
            Self::Err(code) => write!(f, "ERR {}", code.word()),
        }
    }
}

impl FromStr for Reply {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let upper = s.trim_end_matches(['\r', '\n']).to_ascii_uppercase();
        let mut words = upper.split_whitespace();
        let code = |word: &str| ErrCode::ALL.into_iter().find(|c| c.word() == word);
        match (words.next(), words.next(), words.next()) {
            (Some("OK"), None, _) => Ok(Self::Ok),
            (Some("ERR"), Some(w), None) => code(w).map(Self::Err).ok_or(ParseError::Args),
            _ => Err(ParseError::Args),
        }
    }
}
