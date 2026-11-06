//! Versioned TOML config: parsed once into typed values, so bad input fails at load time.

use bd_journal::{
    Event, GameState,
    status::{Flags, Flags2},
};
use bd_protocol::{
    Anim, ClockTime, Counter, FourDigit, LedRange, OutOfRange, PeriodMs, Rgb, Row, Seconds,
    SixDigit, Watch,
};
use serde::Deserialize;
use thiserror::Error;

use crate::template::Template;

/// The only config version this build reads.
pub const VERSION: u32 = 1;

// Single source of truth: the example file doubles as the built-in rules, so they cannot drift.
const DEFAULT_TOML: &str = include_str!("../default.toml");

/// Why a config did not load.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Not valid TOML, or a field has the wrong shape.
    #[error("invalid config: {0}")]
    Parse(#[from] toml::de::Error),
    /// `version` is not one we know.
    #[error("unsupported config version {0}, expected {VERSION}")]
    Version(u32),
    /// One rule has a bad value. `number` counts from 1, like a person would.
    #[error("rule {number}: {source}")]
    Rule {
        /// Position in the file.
        number: usize,
        /// What is wrong.
        source: RuleError,
    },
}

/// A bad value inside one rule.
#[derive(Debug, Error)]
pub enum RuleError {
    /// LED range, row, period, counter or seconds outside the device limits.
    #[error(transparent)]
    Range(#[from] OutOfRange),
    /// Colour is not six hex digits.
    #[error("invalid colour `{0}`, expected RRGGBB")]
    Color(String),
    /// Time is not `HH:MM`.
    #[error("invalid time `{0}`, expected HH:MM")]
    Time(String),
    /// Bad LCD template.
    #[error("invalid LCD text: {0}")]
    Template(String),
    /// Fields that do not go together.
    #[error("{0}")]
    Fields(&'static str),
}

/// What makes a rule fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    /// Level-based: active for as long as the condition holds.
    State(Condition),
    /// Edge-based: fires when this journal event arrives; the output stays until replaced.
    Event(EventKind),
}

/// A condition on the game state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// In a session (between `LoadGame` and `Shutdown`).
    InGame,
    /// Landing gear down.
    GearDown,
    /// Shields are down. Unknown (no `Status.json` yet) counts as not down.
    ShieldsDown,
    /// Hardpoints deployed.
    HardpointsDeployed,
    /// Docked.
    Docked,
    /// FSD charging.
    FsdCharging,
    /// Low fuel, by the game's flag or by main tank under 25%.
    LowFuel,
    /// In supercruise.
    Supercruise,
}

impl Condition {
    /// Whether the condition holds in this state.
    pub fn holds(self, s: &GameState) -> bool {
        let flags = s.status.as_ref().map(|st| (st.flags, st.flags2));
        let flag = |f: Flags| flags.is_some_and(|(fl, _)| fl.contains(f));
        match self {
            Self::InGame => s.in_game,
            Self::GearDown => flag(Flags::LANDING_GEAR_DOWN),
            Self::ShieldsDown => flags.is_some() && !flag(Flags::SHIELDS_UP),
            Self::HardpointsDeployed => flag(Flags::HARDPOINTS_DEPLOYED),
            Self::Docked => s.station.is_some() || flag(Flags::DOCKED),
            // Flags2 has its own hyperdrive bit; either means a jump is charging.
            Self::FsdCharging => {
                flag(Flags::FSD_CHARGING)
                    || flags.is_some_and(|(_, f2)| f2.contains(Flags2::FSD_HYPERDRIVE_CHARGING))
            }
            Self::LowFuel => {
                flag(Flags::LOW_FUEL)
                    || matches!((s.fuel_main, s.fuel_capacity), (Some(m), Some(c)) if c > 0.0 && m < c * 0.25)
            }
            Self::Supercruise => flag(Flags::SUPERCRUISE),
        }
    }
}

/// A journal event kind, named as in the journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)] // names are the journal's own
pub enum EventKind {
    LoadGame,
    Cargo,
    Location,
    FSDJump,
    StartJump,
    SupercruiseEntry,
    SupercruiseExit,
    Docked,
    Undocked,
    Touchdown,
    Liftoff,
    RefuelAll,
    RefuelPartial,
    ReservoirReplenished,
    FuelScoop,
    LaunchSRV,
    Music,
    Died,
    Resurrect,
    Shutdown,
}

impl EventKind {
    /// The kind of an event; `None` for events we do not model.
    pub fn of(event: &Event) -> Option<Self> {
        Some(match event {
            Event::LoadGame { .. } => Self::LoadGame,
            Event::Cargo { .. } => Self::Cargo,
            Event::Location { .. } => Self::Location,
            Event::FSDJump { .. } => Self::FSDJump,
            Event::StartJump { .. } => Self::StartJump,
            Event::SupercruiseEntry { .. } => Self::SupercruiseEntry,
            Event::SupercruiseExit { .. } => Self::SupercruiseExit,
            Event::Docked { .. } => Self::Docked,
            Event::Undocked { .. } => Self::Undocked,
            Event::Touchdown { .. } => Self::Touchdown,
            Event::Liftoff { .. } => Self::Liftoff,
            Event::RefuelAll { .. } => Self::RefuelAll,
            Event::RefuelPartial { .. } => Self::RefuelPartial,
            Event::ReservoirReplenished { .. } => Self::ReservoirReplenished,
            Event::FuelScoop { .. } => Self::FuelScoop,
            Event::LaunchSRV => Self::LaunchSRV,
            Event::Music { .. } => Self::Music,
            Event::Died => Self::Died,
            Event::Resurrect => Self::Resurrect,
            Event::Shutdown => Self::Shutdown,
            Event::Unknown => return None,
        })
    }
}

/// What a rule puts on the board, with every value already validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Light a range of pixels.
    Led {
        /// Pixels.
        range: LedRange,
        /// Colour.
        color: Rgb,
        /// Animation.
        anim: Anim,
    },
    /// Write an LCD row.
    Lcd {
        /// Row.
        row: Row,
        /// Text with placeholders.
        template: Template,
    },
    /// Drive the 4-digit display.
    FourDigit(FourDigit),
    /// Drive the 6-digit display.
    SixDigit(SixDigit),
}

/// One rule: when `trigger` holds, show `action`; higher `priority` wins a shared output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// What makes it fire.
    pub trigger: Trigger,
    /// What it shows.
    pub action: Action,
    /// Higher wins. Ties go to the rule listed first.
    pub priority: i32,
}

/// All rules, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The rules.
    pub rules: Vec<Rule>,
}

impl Default for Config {
    // The file is checked by a test, so a failure here cannot reach users.
    fn default() -> Self {
        Self::from_toml(DEFAULT_TOML).unwrap_or(Self { rules: Vec::new() })
    }
}

impl Config {
    /// Parses and validates a TOML config.
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let raw: RawConfig = toml::from_str(text)?;
        if raw.version != VERSION {
            return Err(ConfigError::Version(raw.version));
        }
        let rules = raw
            .rules
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                r.validate().map_err(|source| ConfigError::Rule {
                    number: i + 1,
                    source,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { rules })
    }
}

// Raw shapes mirror the file; `deny_unknown_fields` turns typos into errors.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    version: u32,
    #[serde(default)]
    rules: Vec<RawRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    trigger: Trigger,
    action: RawAction,
    #[serde(default)]
    priority: i32,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RawAnim {
    #[default]
    Solid,
    Blink,
    Pulse,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RawFour {
    Time,
    Startwatch,
    Timer,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RawSix {
    Set,
    Reset,
    CountUp,
    CountDown,
}

// Numbers are read wide so out-of-range values reach the bd-protocol constructors and
// get its error instead of a generic serde one.
#[derive(Deserialize)]
#[serde(tag = "output", rename_all = "snake_case", deny_unknown_fields)]
enum RawAction {
    Led {
        start: u32,
        end: u32,
        color: String,
        #[serde(default)]
        anim: RawAnim,
        period_ms: Option<u32>,
    },
    Lcd {
        row: u32,
        text: String,
    },
    FourDigit {
        show: RawFour,
        at: Option<String>,
        seconds: Option<u32>,
        #[serde(default)]
        pause: bool,
    },
    SixDigit {
        show: RawSix,
        value: Option<u32>,
        period_ms: Option<u32>,
    },
}

impl RawRule {
    fn validate(self) -> Result<Rule, RuleError> {
        Ok(Rule {
            trigger: self.trigger,
            action: self.action.validate()?,
            priority: self.priority,
        })
    }
}

fn led_pixel(n: u32) -> Result<u8, OutOfRange> {
    u8::try_from(n).map_err(|_| OutOfRange("LED range"))
}

fn clock(text: &str) -> Result<ClockTime, RuleError> {
    let bad = || RuleError::Time(text.to_owned());
    let (h, m) = text.split_once(':').ok_or_else(bad)?;
    let (h, m) = (h.parse().map_err(|_| bad())?, m.parse().map_err(|_| bad())?);
    Ok(ClockTime::new(h, m)?)
}

fn watch(seconds: Option<u32>, pause: bool) -> Result<Watch, RuleError> {
    match (seconds, pause) {
        (None, false) => Ok(Watch::Resume),
        (None, true) => Ok(Watch::Pause),
        (Some(s), false) => Ok(Watch::Start(Seconds::new(s)?)),
        (Some(_), true) => Err(RuleError::Fields(
            "`seconds` and `pause` exclude each other",
        )),
    }
}

fn period(ms: Option<u32>) -> Result<Option<PeriodMs>, OutOfRange> {
    ms.map(PeriodMs::new).transpose()
}

impl RawAction {
    fn validate(self) -> Result<Action, RuleError> {
        match self {
            Self::Led {
                start,
                end,
                color,
                anim,
                period_ms,
            } => {
                let range = LedRange::new(led_pixel(start)?, led_pixel(end)?)?;
                let color = Rgb::from_hex(&color).ok_or(RuleError::Color(color))?;
                let p = period(period_ms)?;
                let anim = match (anim, p) {
                    (RawAnim::Solid, None) => Anim::Solid,
                    (RawAnim::Solid, Some(_)) => {
                        return Err(RuleError::Fields("a solid LED has no `period_ms`"));
                    }
                    (RawAnim::Blink, p) => Anim::Blink(p),
                    (RawAnim::Pulse, p) => Anim::Pulse(p),
                };
                Ok(Action::Led { range, color, anim })
            }
            Self::Lcd { row, text } => Ok(Action::Lcd {
                row: Row::new(row)?,
                template: Template::parse(&text)?,
            }),
            Self::FourDigit {
                show,
                at,
                seconds,
                pause,
            } => Ok(Action::FourDigit(match (show, at) {
                (RawFour::Time, at) if seconds.is_none() && !pause => {
                    FourDigit::Time(at.as_deref().map(clock).transpose()?)
                }
                (RawFour::Startwatch, None) => FourDigit::Startwatch(watch(seconds, pause)?),
                (RawFour::Timer, None) => FourDigit::Timer(watch(seconds, pause)?),
                _ => {
                    return Err(RuleError::Fields(
                        "4-digit: `at` is only for time; `seconds`/`pause` only for startwatch and timer",
                    ));
                }
            })),
            Self::SixDigit {
                show,
                value,
                period_ms,
            } => {
                let p = period(period_ms)?;
                let need = || RuleError::Fields("6-digit: this mode needs `value`");
                Ok(Action::SixDigit(match (show, value, p) {
                    (RawSix::Set, Some(v), None) => SixDigit::Set(Counter::new(v)?),
                    (RawSix::Reset, None, None) => SixDigit::Reset,
                    (RawSix::CountUp, v, period) => SixDigit::CountUp {
                        from: Counter::new(v.ok_or_else(need)?)?,
                        period,
                    },
                    (RawSix::CountDown, v, period) => SixDigit::CountDown {
                        from: Counter::new(v.ok_or_else(need)?)?,
                        period,
                    },
                    _ => {
                        return Err(RuleError::Fields(
                            "6-digit: set needs `value`, reset takes nothing, only counts take `period_ms`",
                        ));
                    }
                }))
            }
        }
    }
}
