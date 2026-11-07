# Biggus Darkus

An Elite Dangerous companion tool: a custom hardware device that shows live game information, driven by a desktop app that reads the game's journal and state files.

## Parts

| Folder | What it is | Status |
|---|---|---|
| [`firmware`](firmware/) | Firmware for the custom device (Arduino Nano, LED strip, TM1637 digit displays, 16×2 LCD, mode button) | Working |
| [`app`](app/) | Desktop app (Rust, Tauri, React) that reads Elite Dangerous journals and state, and sends them to the device over serial. Its GUI sets the serial port, the journal location and the rules for how the device reacts to game events and state. Also has a headless CLI. | Working; tested with the real device and game, bugs still being collected |
| [`pcb`](pcb/) | KiCad design for the device board | In progress |

## How it fits together

```
Elite Dangerous  ->  journal + state files  ->  desktop app  ->  serial commands  ->  device
```

The app turns game events into serial commands using editable rules. The firmware's command protocol is in [`firmware/docs/commands.md`](firmware/docs/commands.md); the app has a typed copy of it, with generated docs, in [`app/docs/protocol.md`](app/docs/protocol.md).

## For developers and AI sessions

- App design, decisions and known gaps: [`app/docs/architecture.md`](app/docs/architecture.md)
- App build, run and packaging: [`app/README.md`](app/README.md)
- Firmware docs: [`firmware/docs/`](firmware/docs/)
- Elite Dangerous journal format (official): <https://elite-journal.readthedocs.io/en/latest/>
