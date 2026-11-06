//! The pipeline: journal lines in, board commands out. Shared by the CLI and the GUI.
//!
//! `step` and `pump` are pure; `tick`, `run_loop` and `run_once` are the thin I/O shell around them.
//! The shell never prints: it tells the caller what happened through a `Report` callback.

// Tests may panic on failure; that is how they fail.
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic))]

use std::{
    io,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    time::Duration,
};

use bd_device::{ConnectionState, Link, Transport, retry_delay};
use bd_journal::{
    GameState, Source, Tailer, apply, apply_status, dir, event::parse_line, status::parse_status,
};
use bd_protocol::{Command, Lcd, LedStrip, Mode};
use bd_rules::{Config, Output, plan};

/// How long to wait for the board's reply; its LED frames block for only a few ms.
pub const REPLY_TIMEOUT: Duration = Duration::from_secs(1);

/// Safety net for missed file-system notifications.
const POLL: Duration = Duration::from_secs(2);

/// Where the game's journal is, or `None` on an unsupported system.
pub fn detect_journal_dir() -> Option<PathBuf> {
    let platform = dir::current_platform()?;
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let xdg = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    let found = dir::candidates(platform, Path::new(&home), xdg.as_deref());
    dir::detect(&found, Path::is_dir)
}

/// Something the caller may want to show. A callback instead of printing keeps the pipeline UI-agnostic.
#[derive(Debug)]
pub enum Report<'a> {
    /// A human-readable line: a command with its reply, a skipped input, a retry.
    Log(String),
    /// The game state after it changed.
    State(&'a GameState),
    /// The board connection changed.
    Connection(ConnectionState),
}

/// What the pipeline remembers between inputs.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// The game as last understood.
    pub state: GameState,
    /// What we believe the board shows, so only changes are sent.
    output: Output,
}

/// One thing read from the game.
#[derive(Debug, Clone, Copy)]
pub enum Input<'a> {
    /// A complete journal line.
    Line(&'a str),
    /// The contents of `Status.json`.
    Status(&'a str),
}

/// Fold one input into the model and say what to send. A bad input leaves the model
/// untouched and comes back as a message, so one broken line never stops the run.
pub fn step(config: &Config, model: Model, input: Input) -> (Model, Result<Vec<Command>, String>) {
    let parsed = match input {
        Input::Line(l) => parse_line(l)
            .map(|e| (Some(e.clone()), apply(model.state.clone(), e)))
            .map_err(|e| format!("skipping bad journal line ({e}): {l}")),
        Input::Status(t) => parse_status(t)
            .map(|s| (None, apply_status(model.state.clone(), s)))
            .map_err(|e| format!("skipping bad Status.json ({e})")),
    };
    match parsed {
        Ok((event, state)) => {
            let (commands, output) = plan(config, &state, event.as_ref(), &model.output);
            (Model { state, output }, Ok(commands))
        }
        Err(msg) => (model, Err(msg)),
    }
}

/// Run a batch of inputs through `step`; returns the commands in order and the skip messages.
pub fn pump(config: &Config, model: Model, inputs: &[Input]) -> (Model, Vec<Command>, Vec<String>) {
    inputs.iter().fold(
        (model, Vec::new(), Vec::new()),
        |(m, mut cmds, mut warns), i| {
            let (m, result) = step(config, m, *i);
            match result {
                Ok(c) => cmds.extend(c),
                Err(w) => warns.push(w),
            }
            (m, cmds, warns)
        },
    )
}

/// Blank the board and repaint it from the state. `Output` starts empty, so it must match a
/// freshly cleared board; used at start and after a reconnect, when the board forgot everything.
pub fn resync(config: &Config, model: Model) -> (Model, Vec<Command>) {
    let (painted, output) = plan(config, &model.state, None, &Output::default());
    let commands = [Command::Lcd(Lcd::Clear), Command::LedStrip(LedStrip::Clear)]
        .into_iter()
        .chain(painted)
        .collect();
    (Model { output, ..model }, commands)
}

/// Send each command and report it with the reply. Failures are reported, not fatal: the link reconnects.
fn send_all<T: Transport, F: FnMut() -> io::Result<T>>(
    link: &mut Link<T, F>,
    commands: &[Command],
    report: &mut impl FnMut(Report),
) {
    for c in commands {
        report(Report::Log(match link.send(c) {
            Ok(reply) => format!("{c} -> {reply}"),
            Err(e) => format!("{c} failed: {e}"),
        }));
    }
}

/// Paint the board from scratch and return the model that matches it.
fn sync<T: Transport, F: FnMut() -> io::Result<T>>(
    config: &Config,
    model: Model,
    link: &mut Link<T, F>,
    report: &mut impl FnMut(Report),
) -> Model {
    let (model, commands) = resync(config, model);
    send_all(link, &commands, report);
    model
}

/// Read what is new from the game, run it through the pipeline and send the result.
pub fn tick<S: Source, T: Transport, F: FnMut() -> io::Result<T>>(
    source: &S,
    tailer: &mut Tailer,
    config: &Config,
    model: Model,
    link: &mut Link<T, F>,
    report: &mut impl FnMut(Report),
) -> io::Result<Model> {
    let lines = tailer.poll(source)?;
    // Status last: it is the newest picture of the ship.
    let status = source.status()?;
    let inputs: Vec<Input> = lines
        .iter()
        .map(|l| Input::Line(l))
        .chain(status.as_deref().map(Input::Status))
        .collect();
    let before = model.state.clone();
    let (model, commands, warnings) = pump(config, model, &inputs);
    warnings.into_iter().for_each(|w| report(Report::Log(w)));
    // Status.json is re-read every tick, so only a real change is worth telling.
    if model.state != before {
        report(Report::State(&model.state));
    }
    send_all(link, &commands, report);
    Ok(model)
}

/// Replay a recorded folder once and stop; nothing will ever change in it.
pub fn run_once<S: Source, T: Transport, F: FnMut() -> io::Result<T>>(
    source: &S,
    config: &Config,
    link: &mut Link<T, F>,
    report: &mut impl FnMut(Report),
) -> io::Result<()> {
    let model = sync(config, Model::default(), link, report);
    tick(source, &mut Tailer::new(), config, model, link, report).map(drop)
}

/// Follow the game until `stop` is set. `wake` carries file-change notifications.
pub fn run_loop<S: Source, T: Transport, F: FnMut() -> io::Result<T>>(
    source: &S,
    config: &Config,
    link: &mut Link<T, F>,
    wake: &Receiver<()>,
    stop: &AtomicBool,
    report: &mut impl FnMut(Report),
) -> io::Result<()> {
    let mut tailer = Tailer::new();
    let mut model = sync(config, Model::default(), link, report);
    let mut shown = link.state();
    report(Report::Connection(shown));
    while !stop.load(Ordering::SeqCst) {
        model = match link.state() {
            ConnectionState::Connected => tick(source, &mut tailer, config, model, link, report)?,
            state => {
                // The board lost its picture while away, so repaint on success.
                let attempt = match state {
                    ConnectionState::Reconnecting { attempt } => attempt,
                    _ => 0,
                };
                report(Report::Log("board not connected, retrying".to_owned()));
                // Waiting on `wake` lets Ctrl-C interrupt the pause.
                let _ = wake.recv_timeout(retry_delay(attempt));
                match link.try_connect() {
                    ConnectionState::Connected => sync(config, model, link, report),
                    _ => model,
                }
            }
        };
        if link.state() != shown {
            shown = link.state();
            report(Report::Connection(shown));
        }
        let _ = wake.recv_timeout(POLL);
    }
    send_all(link, &[Command::Mode(Mode::Demo)], report);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_device::fake::FakeDevice;
    use bd_journal::ReplaySource;
    use bd_protocol::{Anim, Rgb};
    use std::path::Path;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../bd-journal/tests/fixtures");

    fn linked(fake: &FakeDevice) -> Link<FakeDevice, impl FnMut() -> io::Result<FakeDevice>> {
        let fake = fake.clone();
        let mut link = Link::new(move || Ok(fake.clone()), Duration::from_secs(1));
        assert_eq!(link.try_connect(), ConnectionState::Connected);
        link
    }

    #[test]
    fn fixture_replay_drives_the_fake_board() {
        let fake = FakeDevice::new();
        let mut link = linked(&fake);
        let source = ReplaySource::from_dir(Path::new(FIXTURES)).expect("fixtures");
        run_once(&source, &Config::default(), &mut link, &mut |_| {}).expect("run");

        let board = fake.board();
        assert_eq!(board.lcd[0], "Eranin");
        // Status.json (read last) says docked, but the journal left us on a planet: no station name.
        assert_eq!(board.lcd[1], "");
        assert!(board.leds[28].is_some(), "docked pixels");
        // Status.json also has gear down; the default rules light gear (0..6).
        let gear = board.leds[0].expect("gear led");
        assert_eq!(gear.color, Rgb::from_hex("00FF00").expect("rgb"));
        assert_eq!(gear.anim, Anim::Solid);
        assert!(board.leds[6].is_none(), "hardpoints are stowed");
    }

    #[test]
    fn bad_input_is_reported_and_leaves_the_model_alone() {
        let config = Config::default();
        let (model, result) = step(&config, Model::default(), Input::Line("{not json"));
        assert!(result.is_err());
        assert_eq!(model.state, GameState::default());
        let (_, result) = step(&config, model, Input::Status("nope"));
        assert!(result.is_err());
    }

    #[test]
    fn malformed_lines_are_skipped_not_fatal() {
        let name = "Journal.2026-01-02T090000.01.log";
        let good = r#"{ "timestamp":"2026-01-02T09:00:10Z", "event":"LoadGame", "Commander":"HRC-2", "Ship":"FerDeLance", "FuelLevel":3.9, "FuelCapacity":8.0, "GameMode":"Open", "Credits":1, "Loan":0 }"#;
        let loc = r#"{ "timestamp":"2026-01-02T09:00:11Z", "event":"Location", "StarSystem":"Luhman 16", "Body":"Luhman 16 A", "BodyType":"Star", "Docked":false }"#;
        let mut source = ReplaySource::default();
        source.append(
            name,
            format!("{good}\ngarbage\n{{\"event\":\n{loc}\n").as_bytes(),
        );

        let fake = FakeDevice::new();
        let mut link = linked(&fake);
        let model = sync(&Config::default(), Model::default(), &mut link, &mut |_| {});
        let model = tick(
            &source,
            &mut Tailer::new(),
            &Config::default(),
            model,
            &mut link,
            &mut |_| {},
        )
        .expect("tick");

        assert_eq!(model.state.system.as_deref(), Some("Luhman 16"));
        assert_eq!(fake.board().lcd[0], "Luhman 16");
        let lines = Tailer::new().poll(&source).expect("poll");
        let inputs: Vec<Input> = lines.iter().map(|l| Input::Line(l)).collect();
        let (_, _, warnings) = pump(&Config::default(), Model::default(), &inputs);
        assert_eq!(warnings.len(), 2);
    }
}
