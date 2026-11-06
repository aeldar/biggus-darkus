//! The running pipeline: one background thread, started and stopped on request.

use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    thread::{self, JoinHandle},
};

use bd_device::{ConnectionState, Link, SerialTransport};
use bd_journal::{DirSource, watch::watch};
use bd_pipeline::{REPLY_TIMEOUT, detect_journal_dir, run_loop};
use bd_rules::Config;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::dto::{Change, Connection, Game, LogLine, Snapshot, apply};

/// What the UI asks for when it starts the pipeline.
pub struct Request {
    /// Serial port to open.
    pub port: String,
    /// Journal folder; found automatically if `None`.
    pub journal_dir: Option<PathBuf>,
    /// Rules file (TOML); built-in rules if `None`.
    pub config: Option<PathBuf>,
}

/// A live pipeline thread and the handles to stop it.
struct Running {
    stop: Arc<AtomicBool>,
    /// Wakes the loop so it sees `stop` at once instead of after its poll interval.
    wake: Sender<()>,
    thread: JoinHandle<()>,
}

/// Shared app state: the pipeline (if any) and what the UI last heard.
#[derive(Default)]
pub struct Session {
    running: Mutex<Option<Running>>,
    snapshot: Mutex<Snapshot>,
}

/// A poisoned lock only means another thread panicked; the data is still usable.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Session {
    /// What the UI needs to draw itself from scratch.
    pub fn snapshot(&self) -> Snapshot {
        lock(&self.snapshot).clone()
    }

    /// Remember a change and tell the UI about it.
    fn publish(&self, app: &AppHandle, change: Change) {
        {
            let mut snap = lock(&self.snapshot);
            *snap = apply(std::mem::take(&mut *snap), change.clone());
        }
        // A failed emit means the window is gone; nobody is left to tell.
        let _ = match change {
            Change::Connection(c) => c.emit(app),
            Change::Game(g) => g.emit(app),
            Change::Log(l) => LogLine(l).emit(app),
        };
    }

    /// Start the pipeline on its own thread. Fails if one is already running.
    pub fn start(self: &Arc<Self>, app: AppHandle, request: Request) -> Result<(), String> {
        let mut running = lock(&self.running);
        if running.as_ref().is_some_and(|r| !r.thread.is_finished()) {
            return Err("already running".to_owned());
        }
        let stop = Arc::new(AtomicBool::new(false));
        let (wake, rx) = mpsc::channel();
        let (this, flag, tx) = (Arc::clone(self), Arc::clone(&stop), wake.clone());
        self.publish(&app, Change::Game(Game::default()));
        self.publish(&app, Change::Connection(Connection::Connecting));
        let thread = thread::spawn(move || {
            let result = run(&this, &app, &request, &flag, tx, &rx);
            if let Err(e) = result {
                this.publish(&app, Change::Log(format!("error: {e}")));
            }
            this.publish(&app, Change::Connection(Connection::Stopped));
        });
        *running = Some(Running { stop, wake, thread });
        Ok(())
    }

    /// Stop the pipeline and wait for it, so the board is back in demo mode when this returns.
    pub fn stop(&self) {
        if let Some(r) = lock(&self.running).take() {
            r.stop.store(true, Ordering::SeqCst);
            let _ = r.wake.send(());
            let _ = r.thread.join();
        }
    }
}

/// Connect (failing fast on a wrong port), then follow the game until stopped.
fn run(
    session: &Session,
    app: &AppHandle,
    request: &Request,
    stop: &AtomicBool,
    wake_tx: Sender<()>,
    wake_rx: &mpsc::Receiver<()>,
) -> Result<(), String> {
    let config = match &request.config {
        Some(path) => Config::from_toml(
            &std::fs::read_to_string(path).map_err(|e| format!("cannot read config: {e}"))?,
        )
        .map_err(|e| e.to_string())?,
        None => Config::default(),
    };
    let dir = request
        .journal_dir
        .clone()
        .or_else(detect_journal_dir)
        .ok_or("no journal folder found; choose one")?;
    // Kept alive for the whole loop; dropping it stops the notifications.
    let _watcher = watch(&dir, wake_tx).map_err(|e| format!("cannot watch the journal: {e}"))?;
    let port = request.port.clone();
    let mut link = Link::new(move || SerialTransport::open(&port), REPLY_TIMEOUT);
    if link.try_connect() != ConnectionState::Connected {
        return Err(format!("could not connect to {}", request.port));
    }
    run_loop(
        &DirSource(dir),
        &config,
        &mut link,
        wake_rx,
        stop,
        &mut |report| session.publish(app, report.into()),
    )
    .map_err(|e| e.to_string())
}

/// Stop the pipeline when the app exits, so the board does not stay in active mode.
pub fn stop_on_exit(app: &AppHandle) {
    app.state::<Arc<Session>>().stop();
}
