# Biggus Darkus app

Desktop app that reads Elite Dangerous journal files and drives the Biggus Darkus
board (LEDs, LCD, digit displays) over serial. See `PLAN.md` for the design.

## Dev shell

With direnv and nix-direnv: `direnv allow`. Otherwise: `nix develop`.

## Checks

- Inside the shell: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
- All three through nix: `nix flake check`

## Running

`bd-cli run --port /dev/ttyUSB0` follows the game's journal and drives the board until Ctrl-C
(then the board goes back to `MODE DEMO`). Every command and reply is logged to stderr.

- `--journal-dir <path>`: journal folder; found automatically if omitted.
- `--config <file.toml>`: rules file; the built-in rules are used if omitted.
- `--dry-run`: log the commands without opening a port (replaces `--port`).
- `--replay <dir>`: play back a recorded folder once, such as `crates/bd-journal/tests/fixtures`.

Try it without the game or a board:
`cargo run -p bd-cli -- run --dry-run --replay crates/bd-journal/tests/fixtures`
