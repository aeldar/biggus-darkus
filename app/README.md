# Biggus Darkus app

Desktop app that reads Elite Dangerous journal files and drives the Biggus Darkus
board (LEDs, LCD, digit displays) over serial. See `PLAN.md` for the design.

## Dev shell

With direnv and nix-direnv: `direnv allow`. Otherwise: `nix develop`.

## Checks

- Inside the shell: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
- All three through nix: `nix flake check`. It builds the whole workspace, `src-tauri` included, so the
  checks get the Tauri system libraries as inputs and an empty `ui/dist` (the app embeds it at compile time).
- UI checks run in the dev shell, not in `nix flake check`: `cd ui && pnpm install && pnpm exec tsc --noEmit`
  (`pnpm build` runs the type check too). Vendoring pnpm dependencies offline would need a hash that must
  be updated with every lockfile change, which is not worth it yet.

## Desktop app

`cargo tauri dev` (hot reload) or `cargo tauri build --debug --no-bundle` then `target/debug/bd-app`.
The window has four tabs. Stopping, or closing the window, puts the board back in `MODE DEMO`.

- **Status:** port picker with Start/Stop, the live game state and a log of commands and replies.
- **Rules:** edit the rules as a list (trigger, priority, an editor per output type), with add, remove and
  reorder. Every edit is checked by the backend, so errors read like `rule 3: LED range out of range` and
  appear on the rule. Save writes `rules.toml`; "Reset to defaults" deletes it. "Advanced" shows the TOML.
- **Manual:** type a protocol command. It is parsed first (errors show inline), then sent to the board and
  the reply is shown. The hint list comes from `bd-protocol`'s spec; click one to fill the box. Only works
  while connected.
- **Settings:** journal folder (empty = find automatically, "Auto-detect" fills in what it found), the last
  used port, and "start automatically when the app opens". Settings apply at the next Start.

Files live in the OS config folder (`~/.config/dev.biggusdarkus.desktop/` on Linux): `settings.toml` and
`rules.toml` (same format as `bd-cli --config`). Both are versioned. A missing file means defaults.

Saved rules reach a running session by hot swap: the pipeline thread takes a `Control::SetConfig` message,
switches rules and repaints the board, without reconnecting. Manual commands go through the same queue
(`Control::Send`), so only the pipeline thread ever touches the port and nothing races it. The queue is read
once per loop pass, so a manual command can wait up to one poll (2 s) while the board is reconnecting.
A manual command is not remembered by the pipeline: the next rule change that touches those pixels or
rows overwrites it. After a manual `MODE DEMO` or `MODE OFF` the board refuses rule commands (`ERR MODE`)
until `MODE ACTIVE` or a restart.

- `crates/bd-pipeline` holds the pipeline (parse, plan, send, reconnect, control queue) shared by `bd-cli` and `src-tauri`.
  It reports through a callback, so each caller shows it its own way.
- `src-tauri` is thin glue. The UI only sees its DTOs (`dto.rs`, `settings.rs`, `rules.rs`, `manual.rs`) and
  `bd-rules`' `RuleSpec` family (the editable form of a rule, exported with the `specta` feature).
- `ui/src/bindings.ts` is generated from them by `tauri-specta` and checked in. A test fails when it is stale;
  regenerate with `UPDATE_BINDINGS=1 cargo test -p bd-app`.
- Rust is 1.93 because `specta` 2.0.0-rc.26 (used by `tauri-specta`) needs `fmt::from_fn`. `tauri-specta`
  is still a release candidate, so it and `specta` are pinned with `=`.

## Running

`bd-cli run --port /dev/ttyUSB0` follows the game's journal and drives the board until Ctrl-C
(then the board goes back to `MODE DEMO`). Every command and reply is logged to stderr.

- `--journal-dir <path>`: journal folder; found automatically if omitted.
- `--config <file.toml>`: rules file; the built-in rules are used if omitted.
- `--dry-run`: log the commands without opening a port (replaces `--port`).
- `--replay <dir>`: play back a recorded folder once, such as `crates/bd-journal/tests/fixtures`.

Try it without the game or a board:
`cargo run -p bd-cli -- run --dry-run --replay crates/bd-journal/tests/fixtures`
