//! The stored rules and what the rules editor sees. The file is `rules.toml`, the same format
//! `bd-cli --config` reads; a missing file means the built-in rules.

use std::{fs, io, path::Path};

use bd_rules::{Condition, Config, ConfigError, EventKind, Field, RuleSpec};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::settings::write_atomic;

/// What is wrong with a rules edit, for showing next to the rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct RuleProblem {
    /// Which rule, counted from 1; `None` when the problem is not about one rule.
    pub rule: Option<u32>,
    /// The full message, such as `rule 3: LED range out of range`.
    pub message: String,
}

impl RuleProblem {
    /// A problem that belongs to no particular rule, such as a disk error.
    pub fn general(message: impl ToString) -> Self {
        Self {
            rule: None,
            message: message.to_string(),
        }
    }
}

impl From<ConfigError> for RuleProblem {
    fn from(e: ConfigError) -> Self {
        let rule = match &e {
            ConfigError::Rule { number, .. } => u32::try_from(*number).ok(),
            _ => None,
        };
        Self {
            rule,
            message: e.to_string(),
        }
    }
}

/// The choices the editor offers; the UI cannot list enum values by itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct RuleOptions {
    /// State conditions for a state trigger.
    pub conditions: Vec<Condition>,
    /// Journal events for an event trigger.
    pub events: Vec<EventKind>,
    /// Names usable as `{name}` in LCD text.
    pub fields: Vec<String>,
}

/// All the choices.
pub fn options() -> RuleOptions {
    RuleOptions {
        conditions: Condition::ALL.to_vec(),
        events: EventKind::ALL.to_vec(),
        fields: Field::ALL.iter().map(|f| f.name().to_owned()).collect(),
    }
}

/// The stored rules, or the built-in ones when nothing was saved.
pub fn load(path: &Path) -> Result<Config, RuleProblem> {
    match fs::read_to_string(path) {
        Ok(text) => Config::from_toml(&text).map_err(Into::into),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(RuleProblem::general(format!("cannot read rules: {e}"))),
    }
}

/// Checks an edit and returns the config it makes.
pub fn validate(specs: Vec<RuleSpec>) -> Result<Config, RuleProblem> {
    Config::from_specs(specs).map_err(Into::into)
}

/// The TOML an edit would be saved as.
pub fn to_toml(config: &Config) -> Result<String, RuleProblem> {
    config.to_toml().map_err(RuleProblem::general)
}

/// Validates and stores an edit; nothing is written when it is invalid.
pub fn save(path: &Path, specs: Vec<RuleSpec>) -> Result<Config, RuleProblem> {
    let config = validate(specs)?;
    write_atomic(path, &to_toml(&config)?)
        .map_err(|e| RuleProblem::general(format!("cannot save rules: {e}")))?;
    Ok(config)
}

/// Forgets the stored rules, so the built-in ones apply again.
pub fn reset(path: &Path) -> Result<Config, RuleProblem> {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(RuleProblem::general(format!("cannot reset rules: {e}"))),
    }
    Ok(Config::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_rules::{ActionSpec, AnimSpec};

    fn bad_third_rule() -> Vec<RuleSpec> {
        let mut specs = Config::default().to_specs();
        specs[2].action = ActionSpec::Led {
            start: 0,
            end: 99,
            color: "FFFFFF".to_owned(),
            anim: AnimSpec::Solid,
            period_ms: None,
        };
        specs
    }

    #[test]
    fn problems_name_the_rule() {
        let problem = validate(bad_third_rule()).expect_err("invalid");
        assert_eq!(problem.rule, Some(3));
        assert_eq!(problem.message, "rule 3: LED range out of range");
    }

    #[test]
    fn nothing_stored_means_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(
            load(&dir.path().join("rules.toml")).expect("load"),
            Config::default()
        );
    }

    #[test]
    fn saved_rules_load_again_and_reset_forgets_them() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("sub/rules.toml");
        let mut specs = Config::default().to_specs();
        specs.truncate(2);
        let saved = save(&path, specs).expect("save");
        assert_eq!(saved.rules.len(), 2);
        assert_eq!(load(&path).expect("load"), saved);
        assert_eq!(reset(&path).expect("reset"), Config::default());
        assert_eq!(load(&path).expect("load"), Config::default());
        // Resetting twice is fine.
        reset(&path).expect("reset again");
    }

    #[test]
    fn invalid_rules_are_not_written() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("rules.toml");
        assert!(save(&path, bad_third_rule()).is_err());
        assert!(!path.exists());
    }

    #[test]
    fn a_broken_file_is_reported() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("rules.toml");
        fs::write(&path, "version = 1\n[[rules]]\nbogus = 1\n").expect("write");
        assert!(load(&path).is_err());
    }

    #[test]
    fn options_list_everything() {
        let o = options();
        assert_eq!(o.conditions.len(), Condition::ALL.len());
        assert!(o.fields.contains(&"system".to_owned()));
    }
}
