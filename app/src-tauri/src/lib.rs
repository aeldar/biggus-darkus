//! Tauri shell: wires the pipeline to the window. Thin glue; the logic lives in the crates.

// Tests may panic on failure; that is how they fail.
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic))]

mod dto;
mod session;

use std::{path::PathBuf, sync::Arc};

use bd_device::list_ports as device_ports;
use dto::{Connection, Game, LogLine, Port, Snapshot};
use session::{Request, Session, stop_on_exit};
use tauri::{AppHandle, RunEvent, State};
use tauri_specta::{Builder, collect_commands, collect_events};

/// Serial ports on this machine.
#[tauri::command]
#[specta::specta]
async fn list_ports() -> Result<Vec<Port>, String> {
    device_ports()
        .map(|ports| ports.into_iter().map(Port::from).collect())
        .map_err(|e| e.to_string())
}

/// Start following the game and driving the board on `port`.
#[tauri::command]
#[specta::specta]
async fn start(
    app: AppHandle,
    session: State<'_, Arc<Session>>,
    port: String,
    journal_dir: Option<String>,
    config: Option<String>,
) -> Result<(), String> {
    session.start(
        app,
        Request {
            port,
            journal_dir: journal_dir.map(PathBuf::from),
            config: config.map(PathBuf::from),
        },
    )
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

/// Commands and events the UI can use; also the input for the generated bindings.
fn bindings() -> Builder<tauri::Wry> {
    // One TS type per DTO: these types travel both ways with the same shape.
    Builder::new()
        .disable_serde_phases()
        .commands(collect_commands![list_ports, start, stop, get_snapshot])
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
