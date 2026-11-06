# Biggus Darkus app: implementation plan

## Key references

- **Elite Dangerous journal format:** <https://elite-journal.readthedocs.io/en/latest/> — the official
  documentation. It is the source of truth for event names, fields, `Status.json` flags and file
  layout. Follow it when implementing `bd-journal`, and link the relevant page in a comment next to
  each modelled event or flag set.
- **Device serial protocol:** `../biggus-darkus-firmware/docs/commands.md` (until `bd-protocol` takes over as the spec).

## Layout

One Cargo workspace in `biggus-darkus-app/`. The crates form a one-way chain, and only the last one knows about Tauri.

```
biggus-darkus-app/
  flake.nix  flake.lock  .envrc  rust-toolchain.toml
  Cargo.toml                 # workspace, shared lints
  crates/
    bd-protocol/             # typed device language + doc generator (no I/O)
    bd-journal/              # Elite files -> typed events -> GameState (no serial, no GUI)
    bd-device/               # serial link to the board (no game knowledge)
    bd-rules/                # config + GameState -> Commands (pure logic)
    bd-cli/                  # headless runner and debug tool
  src-tauri/                 # thin glue: wires crates together, exposes commands to the GUI
  ui/                        # TypeScript frontend
```

Dependencies:
- `bd-journal` and `bd-device` depend only on `bd-protocol` or nothing.
- `bd-rules` depends on `bd-journal` and `bd-protocol`.
- `src-tauri` and `bd-cli` depend on everything.

Each crate can be tested with no hardware, no game and no window.

## Nix and direnv

- `flake.nix` has a `devShell` with:
  - the Rust toolchain, pinned by `rust-toolchain.toml` (via fenix or rust-overlay);
  - the Tauri v2 Linux libraries: `webkitgtk_4_1`, `gtk3`, `libsoup_3`, `librsvg`, `openssl`, `pkg-config`;
  - `systemd.dev` for `libudev` (needed by `serialport`);
  - `nodejs`, `pnpm` and `cargo-tauri`.
- `.envrc` is just `use flake` (nix-direnv).
- `nix flake check` runs `cargo fmt --check`, `clippy -D warnings` and `cargo test`.
- Serial access on Linux needs the user in the `dialout` group. That is a system setting; ask before changing it.

## Type safety

- Workspace lints: `unsafe_code = forbid`, clippy `unwrap_used`/`expect_used`/`panic` = deny, `missing_docs` = warn.
- Invalid values can't be built. Newtypes check their input in the constructor: `LedRange` (`start < end <= 46`), `Rgb`, `LcdText` (at most 16 characters, ASCII), `Row` (1 or 2), `ClockTime` (HH:MM).
- Frontend is TypeScript with `strict`. TS types are generated from the Rust types (`tauri-specta`), so the GUI and the backend can't drift apart.

## 1. `bd-protocol`: the device language

The Rust types are the one source of truth.

```rust
enum Command { Mode(Mode), FourDigit(FourDigit), SixDigit(SixDigit), Lcd(Lcd), LedStrip(LedStrip) }
enum Reply   { Ok, Err(ErrCode) }       // ErrCode: Mode, Unknown, Args, Range, Line
```

- `Command::encode() -> Result<Line, EncodeError>` produces the wire text. It enforces the 64-character limit.
- `FromStr` parsers exist for `Reply` and for `Command`. The command parser is used by tests and the fake device.
- A `spec()` function does an exhaustive `match` over every command variant and returns its description, argument table, valid-state note and example. Adding a variant without a spec entry fails to compile.
- The `gen-docs` binary renders the spec to Markdown. A golden test fails if the checked-in `docs/protocol.md` is stale.
- Tests:
  - property tests (`proptest`) that parse(encode(x)) == x;
  - every doc example parses and re-encodes to itself;
  - every example from the firmware's `commands.md` is accepted.

## 2. `bd-journal`: reading the game

Follow the official journal documentation (see Key references) for every event and field.

- **Events:** `serde` enums tagged by `"event"`, with an `Unknown` fallback. Only events we use are modelled, so new game updates never break parsing.
- **Files:**
  - `Journal.*.log` (JSON lines) is tailed.
  - `Status.json` is re-read on change. Its `Flags` and `Flags2` become a typed `bitflags` set (gear, shields, hardpoints, docked, FSD charging, low fuel, and so on), with bit meanings taken from the docs.
  - Later: `NavRoute`, `Cargo`, `Market`.
- **Reducer:** `fn apply(GameState, Event) -> GameState` is a pure function. `GameState` holds things like system, station, ship, fuel and cargo count.
- **Tailer:** a file watcher (`notify`) plus an incremental reader that remembers its offset, handles partial lines and switches to a new journal file when one appears.
- **Source trait:** a real directory and a replay of recorded fixture files both implement it.
- **Journal folder:** detect it by platform, including the Proton path on Linux. It can be overridden in the config.

## 3. `bd-device`: talking to the board

- A `Transport` trait over line I/O, with `serialport` as the real implementation and an in-memory one for tests.
- `Device`: send a `Command`, wait for its `Reply` with a timeout, one command in flight at a time. This respects the firmware's 9600 baud and its blocking LED frames.
- On connect it sends `MODE ACTIVE`. On exit it sends `MODE DEMO`.
- Port listing, reconnect after unplug, and a connection-state stream.
- **Fake device:** a small Rust model of the firmware's mode and `ERR` rules. Integration tests run against it, so protocol misuse (such as `ERR MODE`) is caught without hardware.

## 4. `bd-rules`: behaviour

- **Config:** serde types in versioned TOML. Each rule has a trigger (an event, or a state condition like "landing gear down"), an action (a `Command` template) and a priority.
- **Templates:** LCD text such as `{system}` is truncated to 16 characters by the type, not by hand.
- **Resolver:** `fn plan(config, state, event, last_output) -> Vec<Command>`.
  - Rules that target the same output (an LED range, an LCD row) compete by priority.
  - State rules are level-based, so when the condition ends the next lower rule shows again or the output clears.
  - Only changes are sent.
- **Defaults:** a built-in default rule set so it works out of the box.
- **Tests:** table tests of the form "state sequence in, command list out".

## 5. `src-tauri` and `ui`

- **Backend:** owns the config file, runs the pipeline on background tasks, and emits typed events to the UI (connection state, game state, last commands and replies).
- **GUI screens:**
  - Settings: journal folder with auto-detect, serial port picker, connection status.
  - Rules editor.
  - Live game state.
  - Manual command panel for testing the board.
  - Log view.
- **Frontend:** React + Vite + TS strict.

## Milestones

Each one is testable on its own.

| # | Deliverable | Done when |
|---|---|---|
| 0 | Flake, direnv, empty workspace, lints | `nix flake check` passes in a fresh shell |
| 1 | `bd-protocol` and `docs/protocol.md` | round-trip tests and the doc golden test pass |
| 2 | `bd-device` and `bd-cli send` | the real board answers `OK` to typed commands; fake-device tests pass |
| 3 | `bd-journal` | fixture replay produces the expected `GameState`; live tail works with the game |
| 4 | `bd-rules` and defaults | `bd-cli run` drives the board from the game with no GUI |
| 5 | Tauri shell | window shows connection state and live game state |
| 6 | Settings, rules editor, manual panel | everything configurable without editing files |
| 7 | Packaging | AppImage/deb build through nix |

The CLI comes before the GUI on purpose. It proves the whole pipeline against the real board early, and the GUI then only configures something that already works.

## Decisions

1. **Frontend:** React + TS (alternative: Leptos, all Rust but a smaller ecosystem for form-heavy UIs).
2. **Protocol location:** inside the app workspace for now. The firmware's `commands.md` stays hand-written, and a test checks the app's examples against it. If one spec for both sides is wanted later, `bd-protocol` moves to the project root.
3. **Config format:** TOML.
