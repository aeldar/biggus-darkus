//! Incremental journal reading behind a `Source`, so a real folder and recorded fixtures look the same.
//! File naming (`Journal.<datestamp>.<part>.log`): <https://elite-journal.readthedocs.io/en/latest/File%20Format.html>

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Where journal bytes come from.
pub trait Source {
    /// Journal file names, oldest first.
    fn journals(&self) -> io::Result<Vec<String>>;
    /// Bytes of one journal file from `offset` to its current end.
    fn read_from(&self, name: &str, offset: u64) -> io::Result<Vec<u8>>;
    /// Contents of `Status.json`, if the file exists.
    fn status(&self) -> io::Result<Option<String>>;
}

/// A real journal folder.
#[derive(Debug, Clone)]
pub struct DirSource(pub PathBuf);

impl Source for DirSource {
    fn journals(&self) -> io::Result<Vec<String>> {
        let mut names: Vec<String> = fs::read_dir(&self.0)?
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter(|n| is_journal_name(n))
            .collect();
        // The datestamp in the name sorts chronologically as plain text.
        names.sort();
        Ok(names)
    }

    fn read_from(&self, name: &str, offset: u64) -> io::Result<Vec<u8>> {
        let mut file = fs::File::open(self.0.join(name))?;
        file.seek(SeekFrom::Start(offset))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn status(&self) -> io::Result<Option<String>> {
        match fs::read_to_string(self.0.join("Status.json")) {
            Ok(text) => Ok(Some(text)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Recorded files held in memory. Tests grow them with `append` to mimic a running game.
#[derive(Debug, Clone, Default)]
pub struct ReplaySource {
    files: BTreeMap<String, Vec<u8>>,
    status: Option<String>,
}

impl ReplaySource {
    /// Load every journal file and `Status.json` from a folder of fixtures.
    pub fn from_dir(dir: &Path) -> io::Result<Self> {
        let real = DirSource(dir.to_path_buf());
        let files = real
            .journals()?
            .into_iter()
            .map(|n| Ok((n.clone(), real.read_from(&n, 0)?)))
            .collect::<io::Result<_>>()?;
        Ok(Self {
            files,
            status: real.status()?,
        })
    }

    /// Add bytes to a file, creating it if needed.
    pub fn append(&mut self, name: &str, bytes: &[u8]) {
        self.files
            .entry(name.to_string())
            .or_default()
            .extend_from_slice(bytes);
    }

    /// Replace the `Status.json` contents.
    pub fn set_status(&mut self, text: Option<String>) {
        self.status = text;
    }
}

impl Source for ReplaySource {
    fn journals(&self) -> io::Result<Vec<String>> {
        Ok(self.files.keys().cloned().collect())
    }

    fn read_from(&self, name: &str, offset: u64) -> io::Result<Vec<u8>> {
        let bytes = self
            .files
            .get(name)
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let start = usize::try_from(offset).map_or(bytes.len(), |o| o.min(bytes.len()));
        Ok(bytes[start..].to_vec())
    }

    fn status(&self) -> io::Result<Option<String>> {
        Ok(self.status.clone())
    }
}

fn is_journal_name(name: &str) -> bool {
    name.starts_with("Journal.") && name.ends_with(".log")
}

/// Datestamp of a name like `Journal.<datestamp>.<part>.log`; parts of one session share it.
fn session(name: &str) -> &str {
    name.rsplitn(3, '.').nth(2).unwrap_or(name)
}

/// First file of the session that `newest` belongs to.
fn first_part<'a>(names: &'a [String], newest: &'a str) -> &'a str {
    names
        .iter()
        .find(|n| session(n) == session(newest))
        .map_or(newest, String::as_str)
}

/// Split `pending ++ new` into complete lines plus the unfinished tail.
/// The game flushes mid-line sometimes; the tail waits for its newline.
pub fn split_lines(pending: &[u8], new: &[u8]) -> (Vec<String>, Vec<u8>) {
    let all = [pending, new].concat();
    match all.iter().rposition(|&b| b == b'\n') {
        None => (Vec::new(), all),
        Some(last) => {
            let lines = all[..last]
                .split(|&b| b == b'\n')
                .map(|l| String::from_utf8_lossy(l).trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            (lines, all[last + 1..].to_vec())
        }
    }
}

/// Remembers which file we are in and how far we have read.
#[derive(Debug, Clone, Default)]
pub struct Tailer {
    current: Option<String>,
    offset: u64,
    pending: Vec<u8>,
}

impl Tailer {
    /// Start with nothing read: the first poll reads the newest journal from its start,
    /// because `LoadGame` and `Location` at the top are what rebuild the state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Read what is new since the last poll and return the complete lines.
    /// Files that appeared since the last poll are read in order, so none are skipped.
    pub fn poll(&mut self, source: &impl Source) -> io::Result<Vec<String>> {
        let names = source.journals()?;
        let Some(newest) = names.last() else {
            return Ok(Vec::new());
        };
        // First poll: older sessions are history, but the newest session may span several
        // parts and `LoadGame` sits in the first one.
        let current = self
            .current
            .clone()
            .unwrap_or_else(|| first_part(&names, newest).to_string());
        let later = names.iter().filter(|n| **n > current);

        let mut lines = self.read_current(source, &current)?;
        for name in later {
            *self = Self {
                current: Some(name.clone()),
                ..Self::default()
            };
            lines.extend(self.read_current(source, name)?);
        }
        Ok(lines)
    }

    fn read_current(&mut self, source: &impl Source, name: &str) -> io::Result<Vec<String>> {
        // Set here so the first poll also records which file it started in.
        self.current = Some(name.to_string());
        let bytes = source.read_from(name, self.offset)?;
        let (lines, pending) = split_lines(&self.pending, &bytes);
        self.offset += bytes.len() as u64;
        self.pending = pending;
        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "Journal.2026-01-01T100000.01.log";
    const B: &str = "Journal.2026-01-01T100000.02.log";
    const C: &str = "Journal.2026-01-02T090000.01.log";

    #[test]
    fn split_lines_keeps_partial_tail() {
        let (lines, rest) = split_lines(b"", b"one\ntwo\nthr");
        assert_eq!(lines, ["one", "two"]);
        assert_eq!(rest, b"thr");
        let (lines, rest) = split_lines(&rest, b"ee\n");
        assert_eq!(lines, ["three"]);
        assert!(rest.is_empty());
    }

    #[test]
    fn split_lines_handles_crlf_and_blank_lines() {
        let (lines, rest) = split_lines(b"", b"a\r\n\r\nb\n");
        assert_eq!(lines, ["a", "b"]);
        assert!(rest.is_empty());
    }

    #[test]
    fn split_lines_without_newline_returns_everything_pending() {
        let (lines, rest) = split_lines(b"ab", b"cd");
        assert!(lines.is_empty());
        assert_eq!(rest, b"abcd");
    }

    #[test]
    fn empty_source_yields_nothing() {
        let mut t = Tailer::new();
        assert!(t.poll(&ReplaySource::default()).expect("poll").is_empty());
    }

    #[test]
    fn reads_incrementally_without_repeats() {
        let mut src = ReplaySource::default();
        src.append(A, b"l1\nl2\n");
        let mut t = Tailer::new();
        assert_eq!(t.poll(&src).expect("poll"), ["l1", "l2"]);
        assert!(t.poll(&src).expect("poll").is_empty());
        src.append(A, b"l3\n");
        assert_eq!(t.poll(&src).expect("poll"), ["l3"]);
    }

    #[test]
    fn partial_line_waits_for_newline() {
        let mut src = ReplaySource::default();
        src.append(A, b"l1\nl");
        let mut t = Tailer::new();
        assert_eq!(t.poll(&src).expect("poll"), ["l1"]);
        src.append(A, b"2\n");
        assert_eq!(t.poll(&src).expect("poll"), ["l2"]);
    }

    #[test]
    fn first_poll_skips_older_sessions() {
        let mut src = ReplaySource::default();
        src.append(A, b"old\n");
        src.append(C, b"new\n");
        let mut t = Tailer::new();
        assert_eq!(t.poll(&src).expect("poll"), ["new"]);
    }

    #[test]
    fn first_poll_includes_earlier_parts_of_the_newest_session() {
        let mut src = ReplaySource::default();
        src.append(A, b"old\n");
        src.append(B, b"p2\n");
        src.append(C, b"c1\n");
        let mut t = Tailer::new();
        assert_eq!(t.poll(&src).expect("poll"), ["c1"]);
        let mut src = ReplaySource::default();
        src.append(A, b"p1\n");
        src.append(B, b"p2\n");
        assert_eq!(Tailer::new().poll(&src).expect("poll"), ["p1", "p2"]);
    }

    #[test]
    fn switches_to_newer_file_and_drains_the_old_one() {
        let mut src = ReplaySource::default();
        src.append(A, b"a1\n");
        let mut t = Tailer::new();
        assert_eq!(t.poll(&src).expect("poll"), ["a1"]);
        src.append(A, b"a2\n");
        src.append(B, b"b1\n");
        assert_eq!(t.poll(&src).expect("poll"), ["a2", "b1"]);
        src.append(B, b"b2\n");
        assert_eq!(t.poll(&src).expect("poll"), ["b2"]);
    }

    #[test]
    fn several_new_files_are_read_in_order() {
        let mut src = ReplaySource::default();
        src.append(A, b"a1\n");
        let mut t = Tailer::new();
        t.poll(&src).expect("poll");
        src.append(B, b"b1\n");
        src.append(C, b"c1\n");
        assert_eq!(t.poll(&src).expect("poll"), ["b1", "c1"]);
    }

    #[test]
    fn partial_line_of_old_file_is_dropped_on_switch() {
        let mut src = ReplaySource::default();
        src.append(A, b"a1\nhalf");
        let mut t = Tailer::new();
        assert_eq!(t.poll(&src).expect("poll"), ["a1"]);
        src.append(B, b"b1\n");
        assert_eq!(t.poll(&src).expect("poll"), ["b1"]);
    }

    #[test]
    fn ignores_non_journal_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("Cargo.json"), "{}\n").expect("write");
        let src = DirSource(dir.path().to_path_buf());
        assert!(src.journals().expect("list").is_empty());
        assert!(Tailer::new().poll(&src).expect("poll").is_empty());
    }

    #[test]
    fn dir_source_reads_from_offset_and_sorts() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join(C), "c\n").expect("write");
        fs::write(dir.path().join(A), "abcdef\n").expect("write");
        fs::write(dir.path().join("Status.json"), "{}").expect("write");
        fs::write(dir.path().join("Market.json"), "{}").expect("write");
        let src = DirSource(dir.path().to_path_buf());
        assert_eq!(src.journals().expect("list"), [A, C]);
        assert_eq!(src.read_from(A, 3).expect("read"), b"def\n");
        assert_eq!(src.status().expect("status").as_deref(), Some("{}"));
    }

    #[test]
    fn dir_source_without_status_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(
            DirSource(dir.path().to_path_buf()).status().expect("ok"),
            None
        );
    }

    #[test]
    fn dir_source_tails_a_growing_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(A);
        fs::write(&path, "x1\nx").expect("write");
        let src = DirSource(dir.path().to_path_buf());
        let mut t = Tailer::new();
        assert_eq!(t.poll(&src).expect("poll"), ["x1"]);
        fs::write(&path, "x1\nx2\n").expect("write");
        assert_eq!(t.poll(&src).expect("poll"), ["x2"]);
    }
}
