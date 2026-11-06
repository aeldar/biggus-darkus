//! The protocol as data, and its Markdown rendering. This is the source of `docs/protocol.md`.

use crate::{CommandKind, ErrCode, types::*};

/// Documentation for one command kind.
#[derive(Debug, Clone, Copy)]
pub struct Spec {
    /// Short heading.
    pub title: &'static str,
    /// What the command does.
    pub description: &'static str,
    /// Shape of the line; `[x]` is optional.
    pub syntax: &'static str,
    /// Argument name and meaning.
    pub args: &'static [(&'static str, &'static str)],
    /// In which modes the board accepts it.
    pub valid_in: &'static str,
    /// A line that parses and re-encodes to itself.
    pub example: &'static str,
}

const ACTIVE_ONLY: &str = "`ACTIVE` only; otherwise `ERR MODE`";

/// Documentation entry for a kind. Exhaustive, so a new kind will not compile without one.
pub fn spec(kind: CommandKind) -> Spec {
    match kind {
        CommandKind::Mode => Spec {
            title: "Set the mode",
            description: "Switches the board between demo, active and off. The only command that works in every mode.",
            syntax: "MODE <mode>",
            args: &[("mode", "`DEMO`, `ACTIVE` or `OFF`")],
            valid_in: "every mode",
            example: "MODE ACTIVE",
        },
        CommandKind::Time => Spec {
            title: "4-digit clock",
            description: "Switches the 4-digit display to the clock. With a time it sets the clock and runs; without, it resumes the saved time.",
            syntax: "4DIGIT TIME [HH:MM]",
            args: &[("HH:MM", "24-hour time, hour 0-23, minute 0-59")],
            valid_in: ACTIVE_ONLY,
            example: "4DIGIT TIME 12:34",
        },
        CommandKind::Startwatch => Spec {
            title: "4-digit startwatch",
            description: "Switches to the counting-up startwatch. With seconds it starts there; `PAUSE` freezes it; without an argument it resumes.",
            syntax: "4DIGIT STARTWATCH [seconds|PAUSE]",
            args: &[("seconds", "0-359999")],
            valid_in: ACTIVE_ONLY,
            example: "4DIGIT STARTWATCH 90",
        },
        CommandKind::Timer => Spec {
            title: "4-digit timer",
            description: "Switches to the counting-down timer. With seconds it starts there; `PAUSE` freezes it; without an argument it resumes.",
            syntax: "4DIGIT TIMER [seconds|PAUSE]",
            args: &[("seconds", "0-359999")],
            valid_in: ACTIVE_ONLY,
            example: "4DIGIT TIMER 300",
        },
        CommandKind::SixSet => Spec {
            title: "6-digit set",
            description: "Shows a value and stops changing.",
            syntax: "6DIGIT SET <value>",
            args: &[("value", "0-999999")],
            valid_in: ACTIVE_ONLY,
            example: "6DIGIT SET 42",
        },
        CommandKind::SixReset => Spec {
            title: "6-digit reset",
            description: "Shows 0 and stops changing.",
            syntax: "6DIGIT RESET",
            args: &[],
            valid_in: ACTIVE_ONLY,
            example: "6DIGIT RESET",
        },
        CommandKind::CountUp => Spec {
            title: "6-digit count up",
            description: "Counts up from a value. A hidden counter keeps running.",
            syntax: "6DIGIT COUNTUP <from> [step_ms]",
            args: &[("from", "0-999999"), ("step_ms", "1-65535, default 100")],
            valid_in: ACTIVE_ONLY,
            example: "6DIGIT COUNTUP 100 1000",
        },
        CommandKind::CountDown => Spec {
            title: "6-digit count down",
            description: "Counts down from a value and stops at 0.",
            syntax: "6DIGIT COUNTDOWN <from> [step_ms]",
            args: &[("from", "0-999999"), ("step_ms", "1-65535, default 100")],
            valid_in: ACTIVE_ONLY,
            example: "6DIGIT COUNTDOWN 100",
        },
        CommandKind::LcdRow => Spec {
            title: "LCD row",
            description: "Writes text on a row, or clears the row when no text is given. Row 2 is overwritten by the stats in `DEMO` mode.",
            syntax: "LCD ROW <row> [text]",
            args: &[
                ("row", "1 or 2"),
                ("text", "printable ASCII, 1-16 characters, case kept"),
            ],
            valid_in: ACTIVE_ONLY,
            example: "LCD ROW 1 Biggus",
        },
        CommandKind::LcdClear => Spec {
            title: "LCD clear",
            description: "Clears both rows.",
            syntax: "LCD CLEAR",
            args: &[],
            valid_in: ACTIVE_ONLY,
            example: "LCD CLEAR",
        },
        CommandKind::LedSet => Spec {
            title: "LED range",
            description: "Sets every LED in `[start, end)` to a colour and animation, replacing their old state. LEDs outside the range keep theirs.",
            syntax: "LED_STRIP RANGE <start> <end> COLOR <RRGGBB> ANIM <anim> [period_ms]",
            args: &[
                ("start, end", "`start < end <= 46`"),
                ("RRGGBB", "hex colour"),
                (
                    "anim",
                    "`SOLID`, `BLINK` or `PULSE`; `SOLID` takes no period",
                ),
                (
                    "period_ms",
                    "1-65535, default 500 for `BLINK` and 1000 for `PULSE`",
                ),
            ],
            valid_in: ACTIVE_ONLY,
            example: "LED_STRIP RANGE 23 46 COLOR 0000FF ANIM PULSE 1000",
        },
        CommandKind::LedClearRange => Spec {
            title: "LED range off",
            description: "Turns off the LEDs in `[start, end)`.",
            syntax: "LED_STRIP RANGE <start> <end> CLEAR",
            args: &[("start, end", "`start < end <= 46`")],
            valid_in: ACTIVE_ONLY,
            example: "LED_STRIP RANGE 0 23 CLEAR",
        },
        CommandKind::LedClear => Spec {
            title: "LED all off",
            description: "Turns off all LEDs.",
            syntax: "LED_STRIP CLEAR",
            args: &[],
            valid_in: ACTIVE_ONLY,
            example: "LED_STRIP CLEAR",
        },
    }
}

fn render_kind(kind: CommandKind) -> String {
    let s = spec(kind);
    let args: String = s
        .args
        .iter()
        .map(|(name, what)| format!("| `{name}` | {what} |\n"))
        .collect();
    let args = if args.is_empty() {
        "No arguments.\n".to_owned()
    } else {
        format!("| Argument | Values |\n|---|---|\n{args}")
    };
    format!(
        "### {}\n\n{}\n\n```text\n{}\n```\n\n{args}\n- Valid in: {}\n- Example: `{}`\n\n",
        s.title, s.description, s.syntax, s.valid_in, s.example
    )
}

/// The whole of `docs/protocol.md`.
pub fn render_markdown() -> String {
    let replies: String = ErrCode::ALL
        .iter()
        .map(|c| format!("| `ERR {}` | {} |\n", c.word(), c.meaning()))
        .collect();
    let commands: String = CommandKind::ALL.into_iter().map(render_kind).collect();
    format!(
        "<!-- Generated by `cargo run -p bd-protocol --bin gen-docs`. Do not edit by hand. -->\n\n\
# Device protocol\n\n\
Text lines over USB serial at 9600 baud, 8N1. Each command is one line ending in `\\n`; `\\r` is ignored.\n\n\
- Maximum line length is {LINE_MAX} characters (`ERR LINE` above that).\n\
- Device and keyword arguments are case-insensitive. This crate always sends upper case.\n\
- LCD text keeps its case.\n\
- The strip has {LED_COUNT} LEDs; the LCD has 2 rows of {LCD_WIDTH} characters.\n\n\
## Commands\n\n{commands}\
## Replies\n\n\
| Reply | Meaning |\n|---|---|\n| `OK` | Command applied |\n{replies}"
    )
}
