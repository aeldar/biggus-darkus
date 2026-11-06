//! What the UI sees. Small copies of the core types, so the UI contract does not leak internals.

use bd_device::{ConnectionState, PortInfo};
use bd_journal::GameState;
use bd_pipeline::Report;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

/// How many log lines the snapshot keeps.
const LOG_LIMIT: usize = 200;

/// A serial port to pick from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Port {
    /// Path to open.
    pub path: String,
    /// What kind of port it is.
    pub description: String,
}

impl From<PortInfo> for Port {
    fn from(p: PortInfo) -> Self {
        Self {
            path: p.path,
            description: p.description,
        }
    }
}

/// Board connection, plus whether we are trying at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Event)]
#[serde(tag = "kind")]
pub enum Connection {
    /// Not running.
    Stopped,
    /// Started, board not answering yet.
    Connecting,
    /// Board is answering.
    Connected,
    /// The board went away; this many reconnect attempts have failed.
    Reconnecting {
        /// Failed attempts since the loss.
        attempt: u32,
    },
}

impl From<ConnectionState> for Connection {
    fn from(s: ConnectionState) -> Self {
        match s {
            ConnectionState::Disconnected => Self::Stopped,
            ConnectionState::Connected => Self::Connected,
            ConnectionState::Reconnecting { attempt } => Self::Reconnecting { attempt },
        }
    }
}

/// The parts of the game state worth showing.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type, Event)]
pub struct Game {
    /// False before `LoadGame` and after `Shutdown`.
    pub in_game: bool,
    /// Commander name.
    pub commander: Option<String>,
    /// Ship type.
    pub ship: Option<String>,
    /// Current star system.
    pub system: Option<String>,
    /// Current body.
    pub body: Option<String>,
    /// Station we are docked at.
    pub station: Option<String>,
    /// Main tank, tons.
    pub fuel_main: Option<f64>,
    /// Main tank size, tons.
    pub fuel_capacity: Option<f64>,
    /// Cargo units in the hold.
    pub cargo_count: Option<u32>,
    /// Names of the `Status.json` flags that are set, such as "landing gear down".
    pub flags: Vec<String>,
}

impl From<&GameState> for Game {
    fn from(s: &GameState) -> Self {
        Self {
            in_game: s.in_game,
            commander: s.commander.clone(),
            ship: s.ship.clone(),
            system: s.system.clone(),
            body: s.body.clone(),
            station: s.station.clone(),
            fuel_main: s.fuel_main,
            fuel_capacity: s.fuel_capacity,
            cargo_count: s.cargo_count,
            flags: s
                .status
                .iter()
                .flat_map(|st| st.flags.iter_names())
                .map(|(name, _)| name.replace('_', " ").to_lowercase())
                .collect(),
        }
    }
}

/// One command or reply line from the pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Event)]
pub struct LogLine(pub String);

/// Everything the UI needs to draw itself from scratch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Snapshot {
    /// Board connection.
    pub connection: Connection,
    /// Game state.
    pub game: Game,
    /// The most recent log lines, oldest first.
    pub log: Vec<String>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            connection: Connection::Stopped,
            game: Game::default(),
            log: Vec::new(),
        }
    }
}

/// One change to the snapshot; each maps to one event.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// The connection changed.
    Connection(Connection),
    /// The game state changed.
    Game(Game),
    /// A line was logged.
    Log(String),
}

impl From<Report<'_>> for Change {
    fn from(r: Report) -> Self {
        match r {
            Report::Log(l) => Self::Log(l),
            Report::State(s) => Self::Game(s.into()),
            Report::Connection(c) => Self::Connection(c.into()),
        }
    }
}

/// Pure update of the snapshot, so the same change can be emitted and remembered.
pub fn apply(snapshot: Snapshot, change: Change) -> Snapshot {
    match change {
        Change::Connection(connection) => Snapshot {
            connection,
            ..snapshot
        },
        Change::Game(game) => Snapshot { game, ..snapshot },
        Change::Log(line) => {
            let skip = (snapshot.log.len() + 1).saturating_sub(LOG_LIMIT);
            let log = snapshot
                .log
                .into_iter()
                .skip(skip)
                .chain(Some(line))
                .collect();
            Snapshot { log, ..snapshot }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_journal::{Status, status::parse_status};

    #[test]
    fn connection_states_map_one_to_one() {
        assert_eq!(
            Connection::from(ConnectionState::Reconnecting { attempt: 2 }),
            Connection::Reconnecting { attempt: 2 }
        );
        assert_eq!(
            Connection::from(ConnectionState::Connected),
            Connection::Connected
        );
    }

    #[test]
    fn set_flags_become_readable_names() {
        let status: Status = parse_status(r#"{"Flags": 5}"#).expect("status");
        let state = GameState {
            status: Some(status),
            ..GameState::default()
        };
        assert_eq!(Game::from(&state).flags, ["docked", "landing gear down"]);
    }

    #[test]
    fn log_keeps_only_the_newest_lines() {
        let full = (0..LOG_LIMIT).map(|i| i.to_string()).collect();
        let next = apply(
            Snapshot {
                log: full,
                ..Snapshot::default()
            },
            Change::Log("new".to_owned()),
        );
        assert_eq!(next.log.len(), LOG_LIMIT);
        assert_eq!(next.log.first().map(String::as_str), Some("1"));
        assert_eq!(next.log.last().map(String::as_str), Some("new"));
    }

    #[test]
    fn changes_touch_only_their_part() {
        let s = apply(
            Snapshot::default(),
            Change::Connection(Connection::Connected),
        );
        assert_eq!(s.connection, Connection::Connected);
        assert_eq!(s.game, Game::default());
    }
}
