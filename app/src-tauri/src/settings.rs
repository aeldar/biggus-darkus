//! App settings in a small versioned TOML file. Load and save are plain functions of a path.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

/// The only settings file version this build reads.
const VERSION: u32 = 1;

/// What the user can change in the Settings tab, plus the port remembered from the last start.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Settings {
    /// Journal folder; found automatically when `None`.
    pub journal_dir: Option<String>,
    /// Serial port used the last time the pipeline started.
    pub port: Option<String>,
    /// Start the pipeline when the app opens (needs `port`).
    pub auto_start: bool,
}

/// Why settings did not load or save.
#[derive(Debug, Error)]
pub enum SettingsError {
    /// The file cannot be read or written.
    #[error("settings file: {0}")]
    Io(#[from] io::Error),
    /// Not valid TOML, or a field has the wrong shape.
    #[error("invalid settings: {0}")]
    Parse(#[from] toml::de::Error),
    /// Could not turn the settings into TOML.
    #[error("cannot write settings: {0}")]
    Render(#[from] toml::ser::Error),
    /// `version` is not one we know.
    #[error("unsupported settings version {0}, expected {VERSION}")]
    Version(u32),
}

// The version lives in the file, not in `Settings`, so the UI never has to carry it around.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    version: u32,
    #[serde(default)]
    journal_dir: Option<String>,
    #[serde(default)]
    port: Option<String>,
    #[serde(default)]
    auto_start: bool,
}

/// Reads settings from TOML text.
pub fn parse(text: &str) -> Result<Settings, SettingsError> {
    let file: File = toml::from_str(text)?;
    if file.version != VERSION {
        return Err(SettingsError::Version(file.version));
    }
    Ok(Settings {
        journal_dir: file.journal_dir,
        port: file.port,
        auto_start: file.auto_start,
    })
}

/// Writes settings as TOML text.
pub fn render(settings: &Settings) -> Result<String, SettingsError> {
    Ok(toml::to_string(&File {
        version: VERSION,
        journal_dir: settings.journal_dir.clone(),
        port: settings.port.clone(),
        auto_start: settings.auto_start,
    })?)
}

/// Loads settings; a missing file means first run and gives the defaults.
pub fn load(path: &Path) -> Result<Settings, SettingsError> {
    match fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.into()),
    }
}

/// Writes a file so a crash cannot leave half of it: write beside it, then rename over it.
pub fn write_atomic(path: &Path, text: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

/// Saves settings, creating the folder if needed.
pub fn save(path: &Path, settings: &Settings) -> Result<(), SettingsError> {
    Ok(write_atomic(path, &render(settings)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Settings {
        Settings {
            journal_dir: Some("/games/elite".to_owned()),
            port: Some("/dev/ttyUSB0".to_owned()),
            auto_start: true,
        }
    }

    #[test]
    fn text_round_trips() {
        assert_eq!(
            parse(&render(&sample()).expect("render")).expect("parse"),
            sample()
        );
    }

    #[test]
    fn save_then_load_in_a_new_folder() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested/settings.toml");
        save(&path, &sample()).expect("save");
        assert_eq!(load(&path).expect("load"), sample());
        assert!(
            !path.with_extension("tmp").exists(),
            "temp file is renamed away"
        );
    }

    #[test]
    fn missing_file_is_first_run() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(
            load(&dir.path().join("none.toml")).expect("load"),
            Settings::default()
        );
    }

    #[test]
    fn absent_fields_take_defaults() {
        assert_eq!(parse("version = 1").expect("parse"), Settings::default());
    }

    #[test]
    fn bad_files_are_errors_not_defaults() {
        assert!(matches!(
            parse("version = 2"),
            Err(SettingsError::Version(2))
        ));
        assert!(matches!(
            parse("auto_start = true"),
            Err(SettingsError::Parse(_))
        ));
        assert!(matches!(
            parse("version = 1\nbogus = 1"),
            Err(SettingsError::Parse(_))
        ));
        assert!(matches!(parse("not toml ["), Err(SettingsError::Parse(_))));
    }
}
