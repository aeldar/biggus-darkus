//! Tauri shell: wires the pipeline to the window. Thin glue; the logic lives in the crates.

// Tests may panic on failure; that is how they fail.
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic))]

mod dto;
mod manual;
mod rules;
mod session;
mod settings;

use std::{path::PathBuf, sync::Arc};

use bd_device::list_ports as device_ports;
use bd_rules::RuleSpec;
use dto::{Connection, Game, LogLine, Port, Snapshot};
use manual::CommandHelp;
use rules::{RuleOptions, RuleProblem};
use session::{Request, Session, stop_on_exit};
use settings::Settings;
use tauri::{AppHandle, Manager, RunEvent, State};
use tauri_specta::{Builder, collect_commands, collect_events};

/// A file in the OS config folder, such as `~/.config/dev.biggusdarkus.desktop/` on Linux.
fn config_file(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(name))
        .map_err(|e| e.to_string())
}

fn settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    config_file(app, "settings.toml")
}

fn rules_file(app: &AppHandle) -> Result<PathBuf, String> {
    config_file(app, "rules.toml")
}

/// Serial ports on this machine.
#[tauri::command]
#[specta::specta]
async fn list_ports() -> Result<Vec<Port>, String> {
    device_ports()
        .map(|ports| ports.into_iter().map(Port::from).collect())
        .map_err(|e| e.to_string())
}

/// Start the pipeline on `port` with the saved settings and rules, and remember the port.
fn start_on(app: &AppHandle, session: &Arc<Session>, port: String) -> Result<(), String> {
    let saved = settings::load(&settings_file(app)?).map_err(|e| e.to_string())?;
    let config = rules::load(&rules_file(app)?).map_err(|p| p.message)?;
    session.start(
        app.clone(),
        Request {
            port: port.clone(),
            journal_dir: saved.journal_dir.clone().map(PathBuf::from),
            config,
        },
    )?;
    settings::save(
        &settings_file(app)?,
        &Settings {
            port: Some(port),
            ..saved
        },
    )
    .map_err(|e| e.to_string())
}

/// Start following the game and driving the board on `port`.
#[tauri::command]
#[specta::specta]
async fn start(
    app: AppHandle,
    session: State<'_, Arc<Session>>,
    port: String,
) -> Result<(), String> {
    start_on(&app, &session, port)
}

/// Stop and put the board back in demo mode.
#[tauri::command]
#[specta::specta]
async fn stop(session: State<'_, Arc<Session>>) -> Result<(), String> {
    session.stop();
    Ok(())
}

/// Current state, for a window that opened after events were already sent.
#[tauri::command]
#[specta::specta]
fn get_snapshot(session: State<'_, Arc<Session>>) -> Snapshot {
    session.snapshot()
}

/// The saved settings, or the defaults on first run.
#[tauri::command]
#[specta::specta]
fn get_settings(app: AppHandle) -> Result<Settings, String> {
    settings::load(&settings_file(&app)?).map_err(|e| e.to_string())
}

/// Save settings. They apply at the next start.
#[tauri::command]
#[specta::specta]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings::save(&settings_file(&app)?, &settings).map_err(|e| e.to_string())
}

/// Where the game's journal is on this machine, if it can be found.
#[tauri::command]
#[specta::specta]
fn detect_journal_dir() -> Option<String> {
    bd_pipeline::detect_journal_dir().map(|p| p.display().to_string())
}

/// The rules in use: saved ones, or the built-in ones.
#[tauri::command]
#[specta::specta]
fn get_rules(app: AppHandle) -> Result<Vec<RuleSpec>, RuleProblem> {
    let path = rules_file(&app).map_err(RuleProblem::general)?;
    rules::load(&path).map(|c| c.to_specs())
}

/// The choices for triggers and LCD fields.
#[tauri::command]
#[specta::specta]
fn rule_options() -> RuleOptions {
    rules::options()
}

/// Check an edit without saving it. On success returns the TOML it would be saved as.
#[tauri::command]
#[specta::specta]
fn validate_rules(rules: Vec<RuleSpec>) -> Result<String, RuleProblem> {
    rules::validate(rules).and_then(|c| rules::to_toml(&c))
}

/// Save an edit and, if the pipeline is running, switch it to the new rules at once.
#[tauri::command]
#[specta::specta]
fn save_rules(
    app: AppHandle,
    session: State<'_, Arc<Session>>,
    rules: Vec<RuleSpec>,
) -> Result<(), RuleProblem> {
    let path = rules_file(&app).map_err(RuleProblem::general)?;
    let config = rules::save(&path, rules)?;
    session.set_rules(config);
    Ok(())
}

/// Drop the saved rules, apply the built-in ones and return them.
#[tauri::command]
#[specta::specta]
fn reset_to_defaults(
    app: AppHandle,
    session: State<'_, Arc<Session>>,
) -> Result<Vec<RuleSpec>, RuleProblem> {
    let path = rules_file(&app).map_err(RuleProblem::general)?;
    let config = rules::reset(&path)?;
    let specs = config.to_specs();
    session.set_rules(config);
    Ok(specs)
}

/// Every command kind with a syntax line and an example.
#[tauri::command]
#[specta::specta]
fn command_help() -> Vec<CommandHelp> {
    manual::help()
}

/// Parse typed text. Returns the line as it would be sent, or what is wrong with it.
#[tauri::command]
#[specta::specta]
fn check_command(text: String) -> Result<String, String> {
    manual::check(&text).map(|c| c.to_string())
}

/// Send typed text to the board through the running pipeline and return the board's reply.
#[tauri::command]
#[specta::specta]
async fn send_command(session: State<'_, Arc<Session>>, text: String) -> Result<String, String> {
    session.send_manual(manual::check(&text)?)
}

/// Start the pipeline at launch when the user asked for it.
fn auto_start(app: &AppHandle) {
    let session = app.state::<Arc<Session>>();
    let loaded = settings_file(app).and_then(|p| settings::load(&p).map_err(|e| e.to_string()));
    let outcome = match loaded {
        Ok(Settings {
            auto_start: true,
            port: Some(port),
            ..
        }) => start_on(app, &session, port),
        Ok(_) => return,
        Err(e) => Err(e),
    };
    if let Err(e) = outcome {
        session.log(app, format!("auto-start failed: {e}"));
    }
}

/// Commands and events the UI can use; also the input for the generated bindings.
fn bindings() -> Builder<tauri::Wry> {
    // One TS type per DTO: these types travel both ways with the same shape.
    Builder::new()
        .disable_serde_phases()
        .commands(collect_commands![
            list_ports,
            start,
            stop,
            get_snapshot,
            get_settings,
            save_settings,
            detect_journal_dir,
            get_rules,
            rule_options,
            validate_rules,
            save_rules,
            reset_to_defaults,
            command_help,
            check_command,
            send_command
        ])
        .events(collect_events![Connection, Game, LogLine])
}

/// Run the app until the window closes.
pub fn run() {
    let builder = bindings();
    let app = tauri::Builder::default()
        .manage(Arc::new(Session::default()))
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            auto_start(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(|handle, event| {
            if matches!(event, RunEvent::Exit) {
                stop_on_exit(handle);
            }
        }),
        Err(e) => {
            eprintln!("error: cannot start the app: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specta_typescript::Typescript;

    const CHECKED_IN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../ui/src/bindings.ts");

    /// Fails when the UI bindings are stale. Regenerate with `UPDATE_BINDINGS=1 cargo test -p bd-app`.
    #[test]
    fn bindings_are_up_to_date() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fresh = dir.path().join("bindings.ts");
        bindings()
            .export(Typescript::default(), &fresh)
            .expect("export");
        let fresh = std::fs::read_to_string(fresh).expect("read");
        if std::env::var_os("UPDATE_BINDINGS").is_some() {
            std::fs::write(CHECKED_IN, &fresh).expect("write");
        }
        let current = std::fs::read_to_string(CHECKED_IN).unwrap_or_default();
        assert!(
            current == fresh,
            "ui/src/bindings.ts is stale; run `UPDATE_BINDINGS=1 cargo test -p bd-app`"
        );
    }
}
