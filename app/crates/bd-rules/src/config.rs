//! Versioned TOML config: parsed once into typed values, so bad input fails at load time.

use bd_journal::{
    Event, GameState,
    status::{Flags, Flags2},
};
use bd_protocol::{
    Anim, ClockTime, Counter, FourDigit, LedRange, OutOfRange, PeriodMs, Rgb, Row, Seconds,
    SixDigit, Watch,
};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    /// Level-based: active for as long as the condition holds.
    State(Condition),
    /// Edge-based: fires when this journal event arrives; the output stays until replaced.
    Event(EventKind),
}

/// A condition on the game state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
    /// Flight assist off.
    FlightAssistOff,
    /// Ship lights on.
    LightsOn,
    /// Cargo scoop deployed.
    CargoScoopDeployed,
    /// Silent running on.
    SilentRunning,
    /// Night vision on.
    NightVision,
    /// Driving the SRV.
    InSrv,
    /// SRV handbrake on.
    SrvHandbrake,
    /// SRV drive assist on.
    SrvDriveAssist,
    /// SRV high beam on.
    SrvHighBeam,
}

impl Condition {
    /// Every condition, for pickers. Keep in step with the enum.
    pub const ALL: [Self; 17] = [
        Self::InGame,
        Self::GearDown,
        Self::ShieldsDown,
        Self::HardpointsDeployed,
        Self::Docked,
        Self::FsdCharging,
        Self::LowFuel,
        Self::Supercruise,
        Self::FlightAssistOff,
        Self::LightsOn,
        Self::CargoScoopDeployed,
        Self::SilentRunning,
        Self::NightVision,
        Self::InSrv,
        Self::SrvHandbrake,
        Self::SrvDriveAssist,
        Self::SrvHighBeam,
    ];

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
            Self::FlightAssistOff => flag(Flags::FLIGHT_ASSIST_OFF),
            Self::LightsOn => flag(Flags::LIGHTS_ON),
            Self::CargoScoopDeployed => flag(Flags::CARGO_SCOOP_DEPLOYED),
            Self::SilentRunning => flag(Flags::SILENT_RUNNING),
            Self::NightVision => flag(Flags::NIGHT_VISION),
            Self::InSrv => flag(Flags::IN_SRV),
            Self::SrvHandbrake => flag(Flags::SRV_HANDBRAKE),
            Self::SrvDriveAssist => flag(Flags::SRV_DRIVE_ASSIST),
            Self::SrvHighBeam => flag(Flags::SRV_HIGH_BEAM),
        }
    }
}

/// A journal event kind, named as in the journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
    /// Every kind, for pickers. Keep in step with the enum.
    pub const ALL: [Self; 20] = [
        Self::LoadGame,
        Self::Cargo,
        Self::Location,
        Self::FSDJump,
        Self::StartJump,
        Self::SupercruiseEntry,
        Self::SupercruiseExit,
        Self::Docked,
        Self::Undocked,
        Self::Touchdown,
        Self::Liftoff,
        Self::RefuelAll,
        Self::RefuelPartial,
        Self::ReservoirReplenished,
        Self::FuelScoop,
        Self::LaunchSRV,
        Self::Music,
        Self::Died,
        Self::Resurrect,
        Self::Shutdown,
    ];

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
        let file: File = toml::from_str(text)?;
        if file.version != VERSION {
            return Err(ConfigError::Version(file.version));
        }
        Self::from_specs(file.rules)
    }

    /// Validates rules in their editable form; errors name the rule by its position, counted from 1.
    pub fn from_specs(specs: Vec<RuleSpec>) -> Result<Self, ConfigError> {
        let rules = specs
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

    /// The rules in their editable form; the inverse of `from_specs`.
    pub fn to_specs(&self) -> Vec<RuleSpec> {
        self.rules.iter().map(Rule::to_spec).collect()
    }

    /// The config as TOML text that `from_toml` reads back to an equal config.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&File {
            version: VERSION,
            rules: self.to_specs(),
        })
    }
}

/// The file as written: a version and the rules.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    version: u32,
    #[serde(default)]
    rules: Vec<RuleSpec>,
}

// The `*Spec` types are the editable, not yet validated shapes. They mirror the file, so the GUI and
// the TOML share one definition; `deny_unknown_fields` turns typos into errors.

/// A rule as written in the file or edited in the GUI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct RuleSpec {
    /// What makes it fire.
    pub trigger: Trigger,
    /// What it shows.
    pub action: ActionSpec,
    /// Higher wins; defaults to 0.
    #[serde(default)]
    pub priority: i32,
}

/// LED animation as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum AnimSpec {
    /// Steady colour.
    #[default]
    Solid,
    /// On/off.
    Blink,
    /// Fade in/out.
    Pulse,
}

/// 4-digit mode as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum FourShow {
    /// Clock.
    Time,
    /// Count-up stopwatch.
    Startwatch,
    /// Count-down timer.
    Timer,
}

/// 6-digit mode as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum SixShow {
    /// Show a value.
    Set,
    /// Show 0.
    Reset,
    /// Count up.
    CountUp,
    /// Count down.
    CountDown,
}

// Numbers are read wide so out-of-range values reach the bd-protocol constructors and
// get its error instead of a generic serde one.
/// What a rule shows, as written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "output", rename_all = "snake_case", deny_unknown_fields)]
pub enum ActionSpec {
    /// A range of LEDs.
    Led {
        /// First pixel.
        start: u32,
        /// Last pixel, exclusive.
        end: u32,
        /// `RRGGBB`.
        color: String,
        /// Animation; solid if omitted.
        #[serde(default)]
        anim: AnimSpec,
        /// Animation period; not for solid.
        period_ms: Option<u32>,
    },
    /// An LCD row.
    Lcd {
        /// 1 or 2.
        row: u32,
        /// Text with `{field}` placeholders.
        text: String,
    },
    /// The 4-digit display.
    FourDigit {
        /// Mode.
        show: FourShow,
        /// `HH:MM`, for the clock only.
        at: Option<String>,
        /// Start value, for startwatch and timer.
        seconds: Option<u32>,
        /// Freeze, for startwatch and timer.
        #[serde(default)]
        pause: bool,
    },
    /// The 6-digit display.
    SixDigit {
        /// Mode.
        show: SixShow,
        /// Value to show or count from.
        value: Option<u32>,
        /// Step period, for counting.
        period_ms: Option<u32>,
    },
}

impl RuleSpec {
    fn validate(self) -> Result<Rule, RuleError> {
        Ok(Rule {
            trigger: self.trigger,
            action: self.action.validate()?,
            priority: self.priority,
        })
    }
}

impl Rule {
    /// The editable form of this rule.
    pub fn to_spec(&self) -> RuleSpec {
        RuleSpec {
            trigger: self.trigger,
            action: self.action.to_spec(),
            priority: self.priority,
        }
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

impl Action {
    fn to_spec(&self) -> ActionSpec {
        match self {
            Self::Led { range, color, anim } => {
                let (anim, period) = match anim {
                    Anim::Solid => (AnimSpec::Solid, None),
                    Anim::Blink(p) => (AnimSpec::Blink, *p),
                    Anim::Pulse(p) => (AnimSpec::Pulse, *p),
                };
                ActionSpec::Led {
                    start: range.start().into(),
                    end: range.end().into(),
                    color: color.to_string(),
                    anim,
                    period_ms: period.map(|p| p.get().into()),
                }
            }
            Self::Lcd { row, template } => ActionSpec::Lcd {
                row: row.number().into(),
                text: template.source(),
            },
            Self::FourDigit(f) => {
                // Resume is the absence of both `seconds` and `pause`.
                let watch = |w: &Watch| match w {
                    Watch::Resume => (None, false),
                    Watch::Pause => (None, true),
                    Watch::Start(s) => (Some(s.get()), false),
                };
                let (show, at, (seconds, pause)) = match f {
                    FourDigit::Time(t) => (FourShow::Time, t.map(|t| t.to_string()), (None, false)),
                    FourDigit::Startwatch(w) => (FourShow::Startwatch, None, watch(w)),
                    FourDigit::Timer(w) => (FourShow::Timer, None, watch(w)),
                };
                ActionSpec::FourDigit {
                    show,
                    at,
                    seconds,
                    pause,
                }
            }
            Self::SixDigit(d) => {
                let (show, value, period) = match d {
                    SixDigit::Set(v) => (SixShow::Set, Some(v.get()), None),
                    SixDigit::Reset => (SixShow::Reset, None, None),
                    SixDigit::CountUp { from, period } => {
                        (SixShow::CountUp, Some(from.get()), *period)
                    }
                    SixDigit::CountDown { from, period } => {
                        (SixShow::CountDown, Some(from.get()), *period)
                    }
                };
                ActionSpec::SixDigit {
                    show,
                    value,
                    period_ms: period.map(|p| p.get().into()),
                }
            }
        }
    }
}

impl ActionSpec {
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
                    (AnimSpec::Solid, None) => Anim::Solid,
                    (AnimSpec::Solid, Some(_)) => {
                        return Err(RuleError::Fields("a solid LED has no `period_ms`"));
                    }
                    (AnimSpec::Blink, p) => Anim::Blink(p),
                    (AnimSpec::Pulse, p) => Anim::Pulse(p),
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
                (FourShow::Time, at) if seconds.is_none() && !pause => {
                    FourDigit::Time(at.as_deref().map(clock).transpose()?)
                }
                (FourShow::Startwatch, None) => FourDigit::Startwatch(watch(seconds, pause)?),
                (FourShow::Timer, None) => FourDigit::Timer(watch(seconds, pause)?),
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
                    (SixShow::Set, Some(v), None) => SixDigit::Set(Counter::new(v)?),
                    (SixShow::Reset, None, None) => SixDigit::Reset,
                    (SixShow::CountUp, v, period) => SixDigit::CountUp {
                        from: Counter::new(v.ok_or_else(need)?)?,
                        period,
                    },
                    (SixShow::CountDown, v, period) => SixDigit::CountDown {
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
