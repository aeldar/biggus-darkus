//! End to end: recorded files in, `GameState` out. No game needed.
#![allow(clippy::expect_used)]

use bd_journal::event::parse_line;
use bd_journal::status::{Flags, parse_status};
use bd_journal::{GameState, ReplaySource, Source, Tailer, apply, apply_status};
use std::path::Path;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn fold_lines(state: GameState, lines: &[String]) -> GameState {
    lines
        .iter()
        .filter_map(|l| parse_line(l).ok())
        .fold(state, apply)
}

#[test]
fn fixture_replay_produces_expected_state() {
    let src = ReplaySource::from_dir(Path::new(FIXTURES)).expect("fixtures");
    let lines = Tailer::new().poll(&src).expect("poll");
    let state = fold_lines(GameState::default(), &lines);

    assert!(state.in_game, "old session's Shutdown must not leak in");
    assert!(state.alive);
    assert_eq!(state.commander.as_deref(), Some("HRC-2"));
    assert_eq!(state.ship.as_deref(), Some("FerDeLance"));
    assert_eq!(state.system.as_deref(), Some("Eranin"));
    assert_eq!(state.body.as_deref(), Some("Eranin 2"));
    assert_eq!(state.station, None);
    assert_eq!(state.jumping, None);
    assert_eq!(state.cargo_count, Some(7));
    assert_eq!(state.fuel_capacity, Some(8.0));
    assert_eq!(state.fuel_main, Some(7.1));
    assert_eq!(state.fuel_reservoir, Some(0.4));
    assert_eq!(state.music.as_deref(), Some("Supercruise"));
}

#[test]
fn status_fixture_folds_into_state() {
    let src = ReplaySource::from_dir(Path::new(FIXTURES)).expect("fixtures");
    let text = src.status().expect("status").expect("present");
    let state = apply_status(GameState::default(), parse_status(&text).expect("parse"));
    let status = state.status.expect("kept");
    assert!(status.flags.contains(Flags::DOCKED | Flags::SHIELDS_UP));
    assert_eq!(state.fuel_reservoir, Some(0.38));
}

#[test]
fn live_style_replay_matches_one_shot_replay() {
    // Feed the same bytes in small odd-sized chunks, as a running game would.
    let full = ReplaySource::from_dir(Path::new(FIXTURES)).expect("fixtures");
    let expected = fold_lines(
        GameState::default(),
        &Tailer::new().poll(&full).expect("poll"),
    );

    let name = "Journal.2026-01-02T090000.01.log";
    let bytes = full.read_from(name, 0).expect("bytes");
    let mut live = ReplaySource::default();
    let mut tailer = Tailer::new();
    let mut state = GameState::default();
    for chunk in bytes.chunks(37) {
        live.append(name, chunk);
        state = fold_lines(state, &tailer.poll(&live).expect("poll"));
    }
    let second = "Journal.2026-01-02T090000.02.log";
    live.append(second, &full.read_from(second, 0).expect("bytes"));
    state = fold_lines(state, &tailer.poll(&live).expect("poll"));

    assert_eq!(state, expected);
}
