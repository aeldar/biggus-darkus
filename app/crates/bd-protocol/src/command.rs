//! Commands the host sends, their wire text and their parser.

use std::{fmt, str::FromStr};

use thiserror::Error;

use crate::types::*;

/// One validated wire line: at most 64 characters, no line break.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line(String);

impl Line {
    /// The text without the terminating newline.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A command that cannot be sent.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// The firmware answers `ERR LINE` above 64 characters.
    #[error("line is {0} characters, limit is {LINE_MAX}")]
    TooLong(usize),
}

/// Why a line did not parse; each case maps to the `ERR` code the firmware would give.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Line longer than 64 characters.
    #[error("line longer than {LINE_MAX} characters")]
    Line,
    /// Unknown device word.
    #[error("unknown device")]
    Unknown,
    /// Missing, extra or malformed argument.
    #[error("missing or malformed argument")]
    Args,
    /// Number out of bounds.
    #[error(transparent)]
    Range(#[from] OutOfRange),
}

/// Board mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// LED patterns, LCD stats, demo digits.
    Demo,
    /// Accepts hardware commands.
    Active,
    /// Everything blank.
    Off,
}

/// Startwatch or timer argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Watch {
    /// Switch to the mode and resume it.
    Resume,
    /// Switch to the mode and freeze it.
    Pause,
    /// Set the value in seconds and run.
    Start(Seconds),
}

/// `4DIGIT` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FourDigit {
    /// Clock; `None` resumes the saved time.
    Time(Option<ClockTime>),
    /// Count-up stopwatch.
    Startwatch(Watch),
    /// Count-down timer.
    Timer(Watch),
}

/// `6DIGIT` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SixDigit {
    /// Show a value and stop.
    Set(Counter),
    /// Show 0 and stop.
    Reset,
    /// Count up; the step defaults to 100 ms.
    CountUp {
        /// Starting value.
        from: Counter,
        /// Step period.
        period: Option<PeriodMs>,
    },
    /// Count down; the step defaults to 100 ms.
    CountDown {
        /// Starting value.
        from: Counter,
        /// Step period.
        period: Option<PeriodMs>,
    },
}

/// `LCD` commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lcd {
    /// Write a row; `None` clears it.
    Row(Row, Option<LcdText>),
    /// Clear both rows.
    Clear,
}

/// LED animation. Solid has no period, so it cannot be given one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anim {
    /// Steady colour.
    Solid,
    /// On/off; the period defaults to 500 ms.
    Blink(Option<PeriodMs>),
    /// Fade in/out; the period defaults to 1000 ms.
    Pulse(Option<PeriodMs>),
}

/// `LED_STRIP` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedStrip {
    /// Light a range.
    Set {
        /// Pixels to change.
        range: LedRange,
        /// Colour.
        color: Rgb,
        /// Animation.
        anim: Anim,
    },
    /// Turn off a range.
    ClearRange(LedRange),
    /// Turn off all LEDs.
    Clear,
}

/// Any command the board understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `MODE`
    Mode(Mode),
    /// `4DIGIT`
    FourDigit(FourDigit),
    /// `6DIGIT`
    SixDigit(SixDigit),
    /// `LCD`
    Lcd(Lcd),
    /// `LED_STRIP`
    LedStrip(LedStrip),
}

/// Every distinct command form; the docs have one entry per kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    /// `MODE <mode>`
    Mode,
    /// `4DIGIT TIME [HH:MM]`
    Time,
    /// `4DIGIT STARTWATCH [seconds|PAUSE]`
    Startwatch,
    /// `4DIGIT TIMER [seconds|PAUSE]`
    Timer,
    /// `6DIGIT SET`
    SixSet,
    /// `6DIGIT RESET`
    SixReset,
    /// `6DIGIT COUNTUP`
    CountUp,
    /// `6DIGIT COUNTDOWN`
    CountDown,
    /// `LCD ROW`
    LcdRow,
    /// `LCD CLEAR`
    LcdClear,
    /// `LED_STRIP RANGE ... COLOR`
    LedSet,
    /// `LED_STRIP RANGE ... CLEAR`
    LedClearRange,
    /// `LED_STRIP CLEAR`
    LedClear,
}

impl CommandKind {
    /// All kinds, in doc order.
    pub const ALL: [Self; 13] = [
        Self::Mode,
        Self::Time,
        Self::Startwatch,
        Self::Timer,
        Self::SixSet,
        Self::SixReset,
        Self::CountUp,
        Self::CountDown,
        Self::LcdRow,
        Self::LcdClear,
        Self::LedSet,
        Self::LedClearRange,
        Self::LedClear,
    ];
}

impl Mode {
    fn word(self) -> &'static str {
        match self {
            Self::Demo => "DEMO",
            Self::Active => "ACTIVE",
            Self::Off => "OFF",
        }
    }
}

impl Command {
    /// Which doc entry describes this command. Exhaustive, so a new variant must pick a kind.
    pub fn kind(&self) -> CommandKind {
        match self {
            Self::Mode(_) => CommandKind::Mode,
            Self::FourDigit(FourDigit::Time(_)) => CommandKind::Time,
            Self::FourDigit(FourDigit::Startwatch(_)) => CommandKind::Startwatch,
            Self::FourDigit(FourDigit::Timer(_)) => CommandKind::Timer,
            Self::SixDigit(SixDigit::Set(_)) => CommandKind::SixSet,
            Self::SixDigit(SixDigit::Reset) => CommandKind::SixReset,
            Self::SixDigit(SixDigit::CountUp { .. }) => CommandKind::CountUp,
            Self::SixDigit(SixDigit::CountDown { .. }) => CommandKind::CountDown,
            Self::Lcd(Lcd::Row(..)) => CommandKind::LcdRow,
            Self::Lcd(Lcd::Clear) => CommandKind::LcdClear,
            Self::LedStrip(LedStrip::Set { .. }) => CommandKind::LedSet,
            Self::LedStrip(LedStrip::ClearRange(_)) => CommandKind::LedClearRange,
            Self::LedStrip(LedStrip::Clear) => CommandKind::LedClear,
        }
    }

    /// Wire text in canonical upper case, checked against the 64-character limit.
    pub fn encode(&self) -> Result<Line, EncodeError> {
        let text = self.to_string();
        let len = text.chars().count();
        (len <= LINE_MAX)
            .then_some(Line(text))
            .ok_or(EncodeError::TooLong(len))
    }

    // Words are joined by single spaces; the LCD text is one word that may hold spaces.
    fn words(&self) -> Vec<String> {
        let s = |x: &str| x.to_owned();
        let watch = |name: &str, w: &Watch| match w {
            Watch::Resume => vec![s(name)],
            Watch::Pause => vec![s(name), s("PAUSE")],
            Watch::Start(secs) => vec![s(name), secs.get().to_string()],
        };
        let count = |name: &str, from: &Counter, period: &Option<PeriodMs>| {
            [s("6DIGIT"), s(name), from.get().to_string()]
                .into_iter()
                .chain(period.map(|p| p.get().to_string()))
                .collect()
        };
        match self {
            Self::Mode(m) => vec![s("MODE"), s(m.word())],
            Self::FourDigit(FourDigit::Time(t)) => [s("4DIGIT"), s("TIME")]
                .into_iter()
                .chain(t.map(|t| t.to_string()))
                .collect(),
            Self::FourDigit(FourDigit::Startwatch(w)) => [s("4DIGIT")]
                .into_iter()
                .chain(watch("STARTWATCH", w))
                .collect(),
            Self::FourDigit(FourDigit::Timer(w)) => {
                [s("4DIGIT")].into_iter().chain(watch("TIMER", w)).collect()
            }
            Self::SixDigit(SixDigit::Set(v)) => vec![s("6DIGIT"), s("SET"), v.get().to_string()],
            Self::SixDigit(SixDigit::Reset) => vec![s("6DIGIT"), s("RESET")],
            Self::SixDigit(SixDigit::CountUp { from, period }) => count("COUNTUP", from, period),
            Self::SixDigit(SixDigit::CountDown { from, period }) => {
                count("COUNTDOWN", from, period)
            }
            Self::Lcd(Lcd::Row(row, text)) => [s("LCD"), s("ROW"), row.number().to_string()]
                .into_iter()
                .chain(text.as_ref().map(|t| t.as_str().to_owned()))
                .collect(),
            Self::Lcd(Lcd::Clear) => vec![s("LCD"), s("CLEAR")],
            Self::LedStrip(LedStrip::Set { range, color, anim }) => {
                let (name, period) = match anim {
                    Anim::Solid => ("SOLID", None),
                    Anim::Blink(p) => ("BLINK", *p),
                    Anim::Pulse(p) => ("PULSE", *p),
                };
                [
                    s("LED_STRIP"),
                    s("RANGE"),
                    range.start().to_string(),
                    range.end().to_string(),
                    s("COLOR"),
                    color.to_string(),
                    s("ANIM"),
                    s(name),
                ]
                .into_iter()
                .chain(period.map(|p| p.get().to_string()))
                .collect()
            }
            Self::LedStrip(LedStrip::ClearRange(r)) => vec![
                s("LED_STRIP"),
                s("RANGE"),
                r.start().to_string(),
                r.end().to_string(),
                s("CLEAR"),
            ],
            Self::LedStrip(LedStrip::Clear) => vec![s("LED_STRIP"), s("CLEAR")],
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.words().join(" "))
    }
}

// The firmware reads numbers with strtol and answers ERR RANGE when that fails.
fn number(text: &str) -> Result<u32, OutOfRange> {
    text.parse().map_err(|_| OutOfRange("number"))
}

fn period(text: &str) -> Result<PeriodMs, OutOfRange> {
    PeriodMs::new(number(text)?)
}

fn led_range(start: &str, end: &str) -> Result<LedRange, ParseError> {
    let byte = |t: &str| u8::try_from(number(t)?).map_err(|_| OutOfRange("LED range"));
    Ok(LedRange::new(byte(start)?, byte(end)?)?)
}

fn clock(text: &str) -> Result<ClockTime, ParseError> {
    let (h, m) = text.split_once(':').ok_or(ParseError::Args)?;
    let part = |t: &str| t.parse::<u8>().map_err(|_| ParseError::Args);
    Ok(ClockTime::new(part(h)?, part(m)?)?)
}

fn watch(args: &[&str]) -> Result<Watch, ParseError> {
    match args {
        [] => Ok(Watch::Resume),
        ["PAUSE"] => Ok(Watch::Pause),
        [secs] => Ok(Watch::Start(Seconds::new(number(secs)?)?)),
        _ => Err(ParseError::Args),
    }
}

fn counting(args: &[&str]) -> Result<(Counter, Option<PeriodMs>), ParseError> {
    match args {
        [from] => Ok((Counter::new(number(from)?)?, None)),
        [from, p] => Ok((Counter::new(number(from)?)?, Some(period(p)?))),
        _ => Err(ParseError::Args),
    }
}

fn led_strip(args: &[&str]) -> Result<LedStrip, ParseError> {
    match args {
        ["CLEAR"] => Ok(LedStrip::Clear),
        ["RANGE", start, end, "CLEAR"] => Ok(LedStrip::ClearRange(led_range(start, end)?)),
        ["RANGE", start, end, "COLOR", color, "ANIM", anim, rest @ ..] => {
            let range = led_range(start, end)?;
            let color = Rgb::from_hex(color).ok_or(ParseError::Args)?;
            let anim = match (*anim, rest) {
                ("SOLID", []) => Anim::Solid,
                ("BLINK", []) => Anim::Blink(None),
                ("BLINK", [p]) => Anim::Blink(Some(period(p)?)),
                ("PULSE", []) => Anim::Pulse(None),
                ("PULSE", [p]) => Anim::Pulse(Some(period(p)?)),
                _ => return Err(ParseError::Args),
            };
            Ok(LedStrip::Set { range, color, anim })
        }
        _ => Err(ParseError::Args),
    }
}

// LCD text keeps its case and inner spaces, so this command is split by hand instead of by words.
fn lcd(action: &str, rest: &str) -> Result<Lcd, ParseError> {
    match (action.to_ascii_uppercase().as_str(), rest) {
        ("CLEAR", "") => Ok(Lcd::Clear),
        ("ROW", rest) => {
            let (row, text) = rest.split_once(' ').unwrap_or((rest, ""));
            let row = Row::new(number(row).map_err(|_| ParseError::Args)?)
                .map_err(|_| ParseError::Args)?;
            let text = if text.is_empty() {
                None
            } else {
                Some(LcdText::new(text)?)
            };
            Ok(Lcd::Row(row, text))
        }
        _ => Err(ParseError::Args),
    }
}

impl FromStr for Mode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        [Self::Demo, Self::Active, Self::Off]
            .into_iter()
            .find(|m| m.word().eq_ignore_ascii_case(s))
            .ok_or(ParseError::Args)
    }
}

impl FromStr for Command {
    type Err = ParseError;

    fn from_str(line: &str) -> Result<Self, ParseError> {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.chars().count() > LINE_MAX {
            return Err(ParseError::Line);
        }
        let (device, rest) = line.split_once(' ').unwrap_or((line, ""));
        let (action, args) = rest.split_once(' ').unwrap_or((rest, ""));
        let upper = |t: &str| t.to_ascii_uppercase();
        let words: Vec<String> = rest.split_whitespace().map(upper).collect();
        let words: Vec<&str> = words.iter().map(String::as_str).collect();

        match upper(device).as_str() {
            "MODE" => match words.as_slice() {
                [m] => Ok(Self::Mode(m.parse()?)),
                _ => Err(ParseError::Args),
            },
            "LCD" => lcd(action, args).map(Self::Lcd),
            "4DIGIT" => match words.as_slice() {
                ["TIME"] => Ok(Self::FourDigit(FourDigit::Time(None))),
                ["TIME", t] => Ok(Self::FourDigit(FourDigit::Time(Some(clock(t)?)))),
                ["STARTWATCH", a @ ..] => Ok(Self::FourDigit(FourDigit::Startwatch(watch(a)?))),
                ["TIMER", a @ ..] => Ok(Self::FourDigit(FourDigit::Timer(watch(a)?))),
                _ => Err(ParseError::Args),
            },
            "6DIGIT" => match words.as_slice() {
                ["SET", v] => Ok(Self::SixDigit(SixDigit::Set(Counter::new(number(v)?)?))),
                ["RESET"] => Ok(Self::SixDigit(SixDigit::Reset)),
                ["COUNTUP", a @ ..] => counting(a)
                    .map(|(from, period)| Self::SixDigit(SixDigit::CountUp { from, period })),
                ["COUNTDOWN", a @ ..] => counting(a)
                    .map(|(from, period)| Self::SixDigit(SixDigit::CountDown { from, period })),
                _ => Err(ParseError::Args),
            },
            "LED_STRIP" => led_strip(&words).map(Self::LedStrip),
            _ => Err(ParseError::Unknown),
        }
    }
}
