# Biggus Darkus app

Desktop app that reads Elite Dangerous journal files and drives the Biggus Darkus
board (LEDs, LCD, digit displays) over serial. See `docs/architecture.md` for the design and decisions.

## Dev shell

With direnv and nix-direnv: `direnv allow`. Otherwise: `nix develop`.

## Checks

- Inside the shell: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
- All three through nix: `nix flake check`. It builds the whole workspace, `src-tauri` included, so the
  checks get the Tauri system libraries as inputs and an empty `ui/dist` (the app embeds it at compile time).
- UI checks run in the dev shell, not in `nix flake check`: `cd ui && pnpm install && pnpm exec tsc --noEmit`
  (`pnpm build` runs the type check too). The package build uses a pnpm hash that needs updating
  with each lockfile change (see Packaging), so the checks skip it.

## Desktop app

`cargo tauri dev` (hot reload) or `cargo tauri build --debug --no-bundle` then `target/debug/bd-app`.

The dev shell sets `GDK_BACKEND=x11`. On native Wayland the webview gets no screen resolution from GTK and renders a blank, unstyled window. Outside the dev shell, start the app with `GDK_BACKEND=x11`.

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

## Packaging

- Build: `nix build` gives `result/bin/bd-app`, plus the `.desktop` file and icons under `result/share`.
  Run: `nix run`. Install: `nix profile install .` (or add `packages.default` to a NixOS / home-manager config).
- The package runs `cargo tauri build --bundles deb` through `cargo-tauri.hook` and installs the deb's
  contents, so the nix package and the deb ship the same files. The UI is built with pnpm from a hash-pinned
  dependency fetch (`uiDeps` in `flake.nix`) and embedded in the binary.
- **Updating the pnpm hash:** after any change to `ui/package.json` or `ui/pnpm-lock.yaml`, set `hash` in
  `uiDeps` to `pkgs.lib.fakeHash`, run `nix build`, and copy the `got: sha256-...` value from the error.
- **GDK_BACKEND=x11:** see the Desktop app section for why. The nix wrapper sets it, and the `.desktop`
  files start the app with `env GDK_BACKEND=x11` (`src-tauri/bd-app.desktop`).
- **deb:** `cargo tauri build --bundles deb` (in the dev shell) writes
  `target/release/bundle/deb/Biggus Darkus_0.1.0_amd64.deb`. Built and its contents checked with
  `dpkg-deb -c`, but not installed on a Debian system. Only the menu entry sets `GDK_BACKEND=x11`: running
  `bd-app` from a terminal on Wayland needs `GDK_BACKEND=x11 bd-app`.
- **AppImage: not supported.** `cargo tauri build --bundles appimage` downloads `linuxdeploy` AppImages
  and runs them, which fails on NixOS (`failed to run linuxdeploy`). It was not tried elsewhere, and its
  launcher would also need `GDK_BACKEND=x11`, so it is not enabled in `tauri.conf.json`.
- **Icons:** `src-tauri/icons/icon.svg` is the source. Regenerate the PNGs with
  `nix shell nixpkgs#librsvg -c rsvg-convert -w <size> -h <size> icon.svg -o <name>.png`
  for 32 (`32x32.png`), 128 (`128x128.png`), 256 (`256x256.png`) and 512 (`icon.png`).

## Running

`bd-cli run --port /dev/ttyUSB0` follows the game's journal and drives the board until Ctrl-C
(then the board goes back to `MODE DEMO`). Every command and reply is logged to stderr.

- `--journal-dir <path>`: journal folder; found automatically if omitted.
- `--config <file.toml>`: rules file; the built-in rules are used if omitted.
- `--dry-run`: log the commands without opening a port (replaces `--port`).
- `--replay <dir>`: play back a recorded folder once, such as `crates/bd-journal/tests/fixtures`.

Try it without the game or a board:
`cargo run -p bd-cli -- run --dry-run --replay crates/bd-journal/tests/fixtures`
