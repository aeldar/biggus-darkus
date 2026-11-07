# Architecture, specs and decisions

For people and AI sessions picking this project up later. `PLAN.md` is the original plan (now history);
this file is what was actually built and why. Run-and-package details are in `README.md`.

## What the app is

A desktop app for the Biggus Darkus board (an Arduino Nano with a 46-pixel LED strip, a 16x2 LCD, a
4-digit and a 6-digit display; firmware in `../../firmware`). It reads Elite Dangerous
journal files and `Status.json`, turns the game state into board commands using user-editable rules, and
sends them over serial. A GUI sets up the port, the journal folder and the rules.

## Initial specs (from the owner)

- Rust, with Tauri for the GUI.
- Nix shell and direnv for every dev dependency.
- Modular: GUI, journal reading and board communication are separate and testable on their own.
- All code typed and type-safe.
- The board protocol is specified separately, in a type-safe way, and compiles into docs.
- The official journal docs, <https://elite-journal.readthedocs.io/en/latest/>, are the source of truth
  for events and flags, especially in `bd-journal`.
- Frontend: React (chosen over Leptos).
- Work is done in milestones, one commit each.

## Crates

One Cargo workspace. Dependencies point one way; only `src-tauri` knows about Tauri.

```
bd-protocol   typed board language, spec, doc generator          (no I/O)
bd-journal    journal/Status.json -> events -> GameState          (no serial, no GUI)
bd-device     serial transport, Device, Link (reconnect), fake    (no game knowledge)
bd-rules      Config (TOML) + GameState -> Commands               (pure)
bd-pipeline   glue loop shared by CLI and GUI                     (generic over Source + Transport)
bd-cli        headless runner / debug tool
src-tauri     Tauri shell, DTOs, settings and rules files
ui/           React + Vite + TypeScript (strict), pnpm
```

- **bd-protocol:** validated newtypes (`LedRange`, `Rgb`, `LcdText` <= 16 chars, `Row`, `ClockTime`,
  `Seconds`, `Counter`, `PeriodMs`) make invalid commands impossible to build. `Command::encode` enforces
  the 64-character line limit. `spec()` is an exhaustive `match`, so a new command without docs does not
  compile. `cargo run -p bd-protocol --bin gen-docs` writes `docs/protocol.md`; a golden test fails if it
  is stale.
- **bd-journal:** serde `Event` tagged by `"event"` with an `Unknown` fallback (only events we use are
  modelled, so game updates never break parsing). `Flags`/`Flags2` are bitflags. `apply` and
  `apply_status` are pure reducers into `GameState`. `Source` trait with `DirSource` (real folder, tail
  with `notify` plus a 2 s poll as safety net) and `ReplaySource` (recorded fixtures). `dir.rs` finds the
  journal folder per platform, including Proton on Linux.
- **bd-device:** `Transport` trait (`SerialTransport` at 9600 baud, plus fakes). `Device` sends one command
  and waits for `OK`/`ERR ...` with a timeout. `Link` wraps it with a connection state machine
  (`transition`, `retry_delay`) and keeps `last_error`. `FakeDevice` models the firmware's mode and `ERR`
  rules so integration tests run without hardware.
- **bd-rules:** `Config` = list of `Rule { trigger, priority, action }`. Triggers are an event kind or a
  state condition. Actions are LED, LCD, 4-digit or 6-digit output. `plan(config, state, event, output)`
  returns commands plus the new `Output`. Output slots are per LED pixel, per LCD row and per digit
  display; the highest priority wins on a slot, and only changes are sent. `default.toml` is the built-in
  rule set (`include_str!`). LCD templates: `{system} {station} {body} {commander} {ship} {fuel}
  {cargo}`. The editable form is `RuleSpec`/`ActionSpec` (exported to TypeScript via the `specta` feature).
- **bd-pipeline:** `step` and `plan` are pure; `tick`, `run_loop` and `run_once` are the thin I/O shell.
  Everything it does is reported through a `Report` callback (it never prints). `Control::Send` (manual
  command) and `Control::SetConfig` (hot-swap rules) arrive on a channel read once per loop pass, so only
  the pipeline thread ever touches the port.
- **src-tauri:** `session.rs` owns the running pipeline thread; `settings.rs`/`rules.rs` read and write
  `settings.toml`/`rules.toml` in the OS config folder; `dto.rs` holds the UI-facing types; `manual.rs`
  parses typed commands and lists help from the spec.

## Board protocol (summary)

Full text: `docs/protocol.md` (generated) and the firmware's `docs/commands.md` (hand-written). 9600 baud
8N1, one line per command, at most 64 characters. Reply is `OK` or `ERR MODE|UNKNOWN|ARGS|RANGE|LINE`.
Device commands work only in `MODE ACTIVE`. The app sends `MODE ACTIVE` on connect and `MODE DEMO` on
stop or exit.

## Key decisions and the reasons

| Decision | Why |
|---|---|
| CLI before GUI | Proves the whole pipeline on the real board early; the GUI then only configures a working thing. |
| Pure core, thin I/O shell | `step`/`plan` take values and return values, so tests need no board, game or window. |
| `Unknown` event fallback | Journal gains events with each game update; modelling only what we use keeps parsing from breaking. |
| Rules are level-based per output slot | When a condition ends, the next lower rule shows again or the slot clears. Only diffs are sent, which matters at 9600 baud. |
| Config is TOML, versioned | Human-editable; same format for GUI and `bd-cli --config`. Missing file = defaults. |
| TS types generated by `tauri-specta` (checked in) | UI and backend cannot drift. A test fails when `ui/src/bindings.ts` is stale. |
| `tauri-specta` and `specta` pinned with `=` | They are release candidates; they also force Rust 1.93 (`fmt::from_fn`). |
| Only the pipeline thread touches the port | No locking races; manual commands go through the same queue. |
| One `Device::connect` retries `MODE ACTIVE` ~8 s | The Nano resets when the port opens and its boot time varies; one send with a 1 s timeout failed spuriously. |
| `GDK_BACKEND=x11` | On native Wayland this WebKitGTK gets no screen resolution, scales the page by about -1/96 and shows a blank unstyled window. Set in the dev shell, the nix wrapper and the `.desktop` file. |
| Package = deb contents via `cargo-tauri.hook` | The nix package and the deb ship the same files. AppImage is unsupported (`linuxdeploy` fails on NixOS). |
| Workspace lints | `unsafe_code = forbid`; clippy `unwrap_used`, `expect_used`, `panic` = deny; `missing_docs` = warn. Tests opt out with `cfg_attr(test, allow(...))`. |

## Conventions

- Official journal docs first. Link the doc page in a comment next to each modelled event or flag set.
- Comments say why, not what. No error handling or features beyond what is asked.
- Prefer pure functions and immutability.
- Before committing, all of these must pass: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and for UI changes
  `cd ui && pnpm exec tsc --noEmit`. `nix flake check` runs the Rust three. Changing a DTO or command
  means regenerating bindings: `UPDATE_BINDINGS=1 cargo test -p bd-app`. Changing the spec means
  regenerating `docs/protocol.md` with `gen-docs`.
- Changing `ui/pnpm-lock.yaml` means updating the pnpm hash in `flake.nix` (steps in `README.md`).
- Serial access on Linux needs the user in the `dialout` group (a system setting; ask before changing).

## Status

All milestones 0-7 are done and committed. The owner tested the GUI against the real board and a real
game: it reads the journal and state and drives the device from the rules. Known bugs are to be
collected in later sessions.

## Known gaps and traps

- Not tested: live follow loop across unplug/replug, a deb install on real Debian, `nix run` and
  `nix profile install`.
- `bd-journal` does not read `NavRoute.json`, `Cargo.json` or `Market.json` yet.
- Manual commands are not tracked by the pipeline; the next rule change on the same pixels or rows
  overwrites them. A manual `MODE DEMO`/`MODE OFF` makes the board answer `ERR MODE` to rule commands
  until `MODE ACTIVE`.
- Settings apply at the next Start; rules hot-swap while running.
- The journal folder is a plain text box (no folder picker).
- `bd-cli send` does not send `MODE ACTIVE` first and tries once.
- `crates/bd-protocol/tests/firmware_commands.txt` is a manual copy of the firmware's examples; it
  goes stale unnoticed when the firmware docs change.
- The deb sets `GDK_BACKEND=x11` only in the menu entry; from a terminal on Wayland run
  `GDK_BACKEND=x11 bd-app`.
- A React effect must not return a value other than a cleanup function (an expression body with
  `scrollIntoView` once threw on unmount).
