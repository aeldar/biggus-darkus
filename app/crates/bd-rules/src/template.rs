//! LCD text templates such as `{system}`.

use bd_journal::GameState;
use bd_protocol::{LCD_WIDTH, LcdText};

use crate::config::RuleError;

/// A game value a template can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// `{system}`
    System,
    /// `{station}`
    Station,
    /// `{body}`
    Body,
    /// `{commander}`
    Commander,
    /// `{ship}`
    Ship,
    /// `{fuel}`: main tank in tons, one decimal.
    Fuel,
    /// `{cargo}`: cargo units.
    Cargo,
}

impl Field {
    fn from_name(name: &str) -> Result<Self, RuleError> {
        match name {
            "system" => Ok(Self::System),
            "station" => Ok(Self::Station),
            "body" => Ok(Self::Body),
            "commander" => Ok(Self::Commander),
            "ship" => Ok(Self::Ship),
            "fuel" => Ok(Self::Fuel),
            "cargo" => Ok(Self::Cargo),
            _ => Err(RuleError::Template(format!("unknown field `{{{name}}}`"))),
        }
    }

    // A value we do not know yet renders as nothing.
    fn value(self, s: &GameState) -> String {
        let value = match self {
            Self::System => s.system.clone(),
            Self::Station => s.station.clone(),
            Self::Body => s.body.clone(),
            Self::Commander => s.commander.clone(),
            Self::Ship => s.ship.clone(),
            Self::Fuel => s.fuel_main.map(|f| format!("{f:.1}")),
            Self::Cargo => s.cargo_count.map(|c| c.to_string()),
        };
        value.unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Lit(String),
    Var(Field),
}

/// Literal text and `{field}` placeholders, checked when the config loads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template(Vec<Part>);

impl Template {
    /// Splits on `{field}` and checks every literal piece with `LcdText::new`.
    pub fn parse(text: &str) -> Result<Self, RuleError> {
        let mut parts = Vec::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            let (lit, tail) = rest.split_at(open);
            let close = tail
                .find('}')
                .ok_or_else(|| RuleError::Template("unclosed `{`".into()))?;
            parts.push(Part::Lit(lit.to_owned()));
            parts.push(Part::Var(Field::from_name(&tail[1..close])?));
            rest = &tail[close + 1..];
        }
        parts.push(Part::Lit(rest.to_owned()));
        // Empty pieces are fine between placeholders, but not a template with nothing in it.
        let parts: Vec<Part> = parts
            .into_iter()
            .filter(|p| !matches!(p, Part::Lit(l) if l.is_empty()))
            .collect();
        if parts.is_empty() {
            LcdText::new("").map_err(|e| RuleError::Template(format!("empty text: {e}")))?;
        }
        parts.iter().try_for_each(|p| match p {
            Part::Lit(l) => LcdText::new(l)
                .map(drop)
                .map_err(|e| RuleError::Template(format!("`{l}`: {e} (1-16 printable ASCII)"))),
            Part::Var(_) => Ok(()),
        })?;
        Ok(Self(parts))
    }

    /// Fills the placeholders. `None` means the row should be blank.
    ///
    /// The result is cut to 16 characters here because names such as star systems and
    /// stations are often longer than the LCD row and the firmware rejects longer text
    /// (`ERR RANGE`). Characters the LCD cannot show become `?`.
    pub fn render(&self, state: &GameState) -> Option<LcdText> {
        let text: String = self
            .0
            .iter()
            .map(|p| match p {
                Part::Lit(l) => l.clone(),
                Part::Var(f) => f.value(state),
            })
            .collect::<String>()
            .chars()
            .map(|c| if (' '..='~').contains(&c) { c } else { '?' })
            .take(LCD_WIDTH)
            .collect();
        LcdText::new(&text).ok()
    }
}
