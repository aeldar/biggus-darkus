//! Wake-ups when the journal folder changes, so callers poll on events instead of on a timer.

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;

/// Send `()` on `tx` for every change in `dir`. Dropping the returned watcher stops it.
/// Callers poll the `Tailer` and re-read `Status.json` on each wake-up; we do not say which file changed
/// because both are cheap to check.
pub fn watch(dir: &Path, tx: Sender<()>) -> notify::Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        // A closed receiver just means nobody listens any more.
        if res.is_ok() {
            let _ = tx.send(());
        }
    })?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn file_write_wakes_the_receiver() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (tx, rx) = mpsc::channel();
        let _w = watch(dir.path(), tx).expect("watch");
        std::fs::write(dir.path().join("Status.json"), "{}").expect("write");
        assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok());
    }
}
