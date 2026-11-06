# Biggus Darkus app

Desktop app that reads Elite Dangerous journal files and drives the Biggus Darkus
board (LEDs, LCD, digit displays) over serial. See `PLAN.md` for the design.

## Dev shell

With direnv and nix-direnv: `direnv allow`. Otherwise: `nix develop`.

## Checks

- Inside the shell: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
- All three through nix: `nix flake check`
