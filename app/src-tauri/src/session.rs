//! The running pipeline: one background thread, started and stopped on request.

use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use bd_device::{ConnectionState, Link, SerialTransport};
use bd_journal::{DirSource, watch::watch};
use bd_pipeline::{Control, REPLY_TIMEOUT, detect_journal_dir, run_loop};
use bd_protocol::Command;
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
    /// Rules to start with.
    pub config: Config,
}

/// A live pipeline thread and the handles to stop it.
struct Running {
    stop: Arc<AtomicBool>,
    /// Wakes the loop so it sees `stop` at once instead of after its poll interval.
    wake: Sender<()>,
    /// Requests for the loop: manual commands and new rules.
    control: Sender<Control>,
    thread: JoinHandle<()>,
}

/// Shared app state: the pipeline (if any) and what the UI last heard.
#[derive(Default)]
pub struct Session {
    running: Mutex<Option<Running>>,
    snapshot: Mutex<Snapshot>,
}

/// How long a manual command may wait for its answer.
const ANSWER_WAIT: Duration = Duration::from_secs(5);

/// A poisoned lock only means another thread panicked; the data is still usable.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Session {
    /// What the UI needs to draw itself from scratch.
    pub fn snapshot(&self) -> Snapshot {
        lock(&self.snapshot).clone()
    }

    /// Add a line to the log, for things that happen outside the pipeline.
    pub fn log(&self, app: &AppHandle, line: String) {
        self.publish(app, Change::Log(line));
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
        let (control, queue) = mpsc::channel();
        let (this, flag, tx) = (Arc::clone(self), Arc::clone(&stop), wake.clone());
        self.publish(&app, Change::Game(Game::default()));
        self.publish(&app, Change::Connection(Connection::Connecting));
        let thread = thread::spawn(move || {
            let result = run(&this, &app, request, &flag, tx, (&rx, &queue));
            if let Err(e) = result {
                this.publish(&app, Change::Log(format!("error: {e}")));
            }
            this.publish(&app, Change::Connection(Connection::Stopped));
        });
        *running = Some(Running {
            stop,
            wake,
            control,
            thread,
        });
        Ok(())
    }

    /// Put a request on the loop's queue and wake it. `None` when no pipeline is running.
    fn enqueue(&self, request: Control) -> Option<()> {
        let running = lock(&self.running);
        let r = running.as_ref().filter(|r| !r.thread.is_finished())?;
        // The loop owns the port, so a request is handled between its steps and cannot race it.
        r.control.send(request).ok()?;
        r.wake.send(()).ok()
    }

    /// Send one command through the running pipeline and wait for the board's reply.
    pub fn send_manual(&self, command: Command) -> Result<String, String> {
        let (answer, reply) = mpsc::channel();
        self.enqueue(Control::Send { command, answer })
            .ok_or("not running")?;
        // A reply takes REPLY_TIMEOUT at most; the margin covers the loop finishing its tick first.
        reply
            .recv_timeout(ANSWER_WAIT)
            .map_err(|_| "no answer from the pipeline".to_owned())?
            .map(|r| r.to_string())
    }

    /// Give a running pipeline new rules; with none running they simply load at the next start.
    pub fn set_rules(&self, config: Config) {
        let _ = self.enqueue(Control::SetConfig(config));
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
    request: Request,
    stop: &AtomicBool,
    wake_tx: Sender<()>,
    (wake_rx, queue): (&Receiver<()>, &Receiver<Control>),
) -> Result<(), String> {
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
        request.config,
        &mut link,
        wake_rx,
        queue,
        stop,
        &mut |report| session.publish(app, report.into()),
    )
    .map_err(|e| e.to_string())
}

/// Stop the pipeline when the app exits, so the board does not stay in active mode.
pub fn stop_on_exit(app: &AppHandle) {
    app.state::<Arc<Session>>().stop();
}
