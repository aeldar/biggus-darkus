# Tasks

Open work for the app. Tick items off (or delete them) when done. Context for each is in
`docs/architecture.md`, "Known gaps and traps".

## Bugs

- [ ] Collect the bugs the owner found while playing, and fix them.

## To test

- [ ] Live follow loop in a real game session: file watcher, new journal file on game restart, Ctrl-C.
- [ ] Reconnect after unplugging and re-plugging the USB cable.
- [ ] GUI Start button on a slow-booting board (connect can take up to about 8 s).
- [ ] Click through the Rules, Manual and Settings tabs and check every action type.
- [ ] Install the `.deb` on a real Debian system.
- [ ] `nix run` and `nix profile install .`.

## To improve

- [ ] Read `NavRoute.json`, `Cargo.json` and `Market.json` in `bd-journal` (official docs first).
- [ ] Track manual commands in the pipeline, so rule changes do not silently overwrite them.
- [ ] Let the Manual tab recover from a manual `MODE DEMO` / `MODE OFF` (rule commands get `ERR MODE`
      until `MODE ACTIVE`).
- [ ] Journal folder picker in Settings (now a plain text box).
- [ ] Make `bd-cli send` send `MODE ACTIVE` first and retry, like the GUI.
- [ ] Replace the manual copy `crates/bd-protocol/tests/firmware_commands.txt` with something that
      cannot go stale unnoticed when the firmware docs change.
- [ ] Set `GDK_BACKEND=x11` for the deb when started from a terminal, not only from the menu entry.
- [ ] Apply Settings without a restart of the session (they apply at the next Start now).
- [ ] AppImage packaging (fails on NixOS because `linuxdeploy` cannot run; try on another system).
