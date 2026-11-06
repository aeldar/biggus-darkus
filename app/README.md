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
The window has a port picker with Start/Stop, the live game state and a log of commands and replies.
Stopping, or closing the window, puts the board back in `MODE DEMO`.

- `crates/bd-pipeline` holds the pipeline (parse, plan, send, reconnect) shared by `bd-cli` and `src-tauri`.
  It reports through a callback, so each caller shows it its own way.
- `src-tauri` is thin glue. Its DTOs (`dto.rs`) are the only types the UI sees.
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
