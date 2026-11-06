# Biggus Darkus app: notes for AI sessions

Desktop app (Rust workspace + Tauri + React) that turns Elite Dangerous journal data into commands for the
Biggus Darkus board. Start with these:

- `docs/architecture.md`: crates, decisions and reasons, conventions, known gaps. Read this first.
- `README.md`: how to build, run, package and test.
- `docs/protocol.md`: the board protocol (generated; do not edit by hand).
- `PLAN.md`: the original plan, kept as history.
- `TASKS.md`: open bugs, things to test and improvements. Update it as work is done.
- `../biggus-darkus-firmware/docs/`: the firmware side (commands, architecture, hardware).
- Journal format (official, source of truth): <https://elite-journal.readthedocs.io/en/latest/>

Rules that matter most:
- Work inside `nix develop` / direnv. Run fmt, clippy `-D warnings` and tests before every commit.
- Regenerate `ui/src/bindings.ts` and `docs/protocol.md` when their sources change (see architecture.md).
- Keep crate dependencies one-way; only `src-tauri` may use Tauri.
- Confirm with the owner before pushing, or changing system state (for example the `dialout` group).
