//! Validated values. Fields are private so an invalid value cannot be built.

use std::fmt;
use thiserror::Error;

/// Pixels on the LED strip: two runs of 23 (hardware.md).
pub const LED_COUNT: u8 = 46;
/// Characters on one LCD row.
pub const LCD_WIDTH: usize = 16;
/// Longest line the firmware accepts, without the newline.
pub const LINE_MAX: usize = 64;
/// Largest value the 6-digit display can show.
pub const COUNTER_MAX: u32 = 999_999;
/// Largest startwatch/timer value, 99:59:59.
pub const SECONDS_MAX: u32 = 359_999;

/// A value outside what the device accepts (`ERR RANGE` on the wire).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{0} out of range")]
pub struct OutOfRange(pub &'static str);

// One place for the "0..=max" check shared by the numeric newtypes.
fn bounded(what: &'static str, value: u32, max: u32) -> Result<u32, OutOfRange> {
    (value <= max).then_some(value).ok_or(OutOfRange(what))
}

/// LED range `[start, end)` with `start < end <= 46`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedRange {
    start: u8,
    end: u8,
}

impl LedRange {
    /// Checks the bounds the firmware checks.
    pub fn new(start: u8, end: u8) -> Result<Self, OutOfRange> {
        (start < end && end <= LED_COUNT)
            .then_some(Self { start, end })
            .ok_or(OutOfRange("LED range"))
    }
    /// First pixel, inclusive.
    pub fn start(self) -> u8 {
        self.start
    }
    /// Last pixel, exclusive.
    pub fn end(self) -> u8 {
        self.end
    }
}

/// A colour, written `RRGGBB` on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl Rgb {
    /// Parses exactly six hex digits, either case.
    pub fn from_hex(text: &str) -> Option<Self> {
        // The digit check also rejects a leading `+` that `from_str_radix` allows.
        let ok = text.len() == 6 && text.bytes().all(|c| c.is_ascii_hexdigit());
        let byte = |i: usize| u8::from_str_radix(&text[i..i + 2], 16).ok();
        ok.then(|| {
            Some(Self {
                r: byte(0)?,
                g: byte(2)?,
                b: byte(4)?,
            })
        })?
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Non-empty printable ASCII, at most 16 characters (one LCD row).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LcdText(String);

impl LcdText {
    /// Rejects empty, too long or non-printable text. An empty row is `None` in the command.
    pub fn new(text: &str) -> Result<Self, OutOfRange> {
        let ok = (1..=LCD_WIDTH).contains(&text.len())
            && text.bytes().all(|c| (0x20..0x7f).contains(&c));
        ok.then(|| Self(text.to_owned()))
            .ok_or(OutOfRange("LCD text"))
    }
    /// The text as sent.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// LCD row, 1 or 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    /// Top row.
    One,
    /// Bottom row.
    Two,
}

impl Row {
    /// Maps 1 or 2 to a row.
    pub fn new(n: u32) -> Result<Self, OutOfRange> {
        match n {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(OutOfRange("LCD row")),
        }
    }
    /// The number sent on the wire.
    pub fn number(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
        }
    }
}

/// 24-hour clock time, `HH:MM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockTime {
    hour: u8,
    minute: u8,
}

impl ClockTime {
    /// Hour 0..=23, minute 0..=59.
    pub fn new(hour: u8, minute: u8) -> Result<Self, OutOfRange> {
        (hour < 24 && minute < 60)
            .then_some(Self { hour, minute })
            .ok_or(OutOfRange("clock time"))
    }
    /// Hour of day.
    pub fn hour(self) -> u8 {
        self.hour
    }
    /// Minute of hour.
    pub fn minute(self) -> u8 {
        self.minute
    }
}

impl fmt::Display for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// Startwatch or timer value in seconds, at most 359999.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seconds(u32);

impl Seconds {
    /// Checks the upper bound.
    pub fn new(value: u32) -> Result<Self, OutOfRange> {
        bounded("seconds", value, SECONDS_MAX).map(Self)
    }
    /// The raw number.
    pub fn get(self) -> u32 {
        self.0
    }
}

/// 6-digit display value, at most 999999.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counter(u32);

impl Counter {
    /// Checks the upper bound.
    pub fn new(value: u32) -> Result<Self, OutOfRange> {
        bounded("counter", value, COUNTER_MAX).map(Self)
    }
    /// The raw number.
    pub fn get(self) -> u32 {
        self.0
    }
}

/// Step or animation period in milliseconds, 1..=65535 (the firmware stores a `u16`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodMs(u16);

impl PeriodMs {
    /// Rejects zero and anything above `u16::MAX`.
    pub fn new(value: u32) -> Result<Self, OutOfRange> {
        u16::try_from(value)
            .ok()
            .filter(|&v| v > 0)
            .map(Self)
            .ok_or(OutOfRange("period"))
    }
    /// The raw number.
    pub fn get(self) -> u16 {
        self.0
    }
}
