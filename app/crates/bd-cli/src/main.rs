//! Headless runner and debug tool.

// Tests may panic on failure; that is how they fail.
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic))]

mod run;

use std::{
    io,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use bd_device::{ConnectionState, Device, Link, SerialTransport, Transport, list_ports};
use bd_journal::{DirSource, ReplaySource, dir, watch::watch};
use bd_protocol::{Command, ParseError, Reply};
use bd_rules::Config;
use clap::{Parser, Subcommand};
use thiserror::Error;

/// How long to wait for the board's reply; its LED frames block for only a few ms.
const REPLY_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Parser)]
#[command(about = "Biggus Darkus board tool")]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// List serial ports.
    Ports,
    /// Send one command and print the reply.
    Send {
        /// Serial port, such as /dev/ttyUSB0.
        #[arg(long)]
        port: String,
        /// The command, such as: LCD ROW 1 Hello
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    /// Follow the game and drive the board until Ctrl-C.
    Run {
        /// Serial port, such as /dev/ttyUSB0.
        #[arg(long, required_unless_present = "dry_run", conflicts_with = "dry_run")]
        port: Option<String>,
        /// Journal folder; found automatically if omitted.
        #[arg(long, conflicts_with = "replay")]
        journal_dir: Option<PathBuf>,
        /// Rules file (TOML); built-in rules if omitted.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Log the commands without opening a port.
        #[arg(long)]
        dry_run: bool,
        /// Play back a recorded journal folder once instead of following the game.
        #[arg(long)]
        replay: Option<PathBuf>,
    },
}

#[derive(Debug, Error)]
enum CliError {
    #[error("invalid command: {0}")]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Device(#[from] bd_device::Error),
    #[error("cannot open port: {0}")]
    Port(#[from] std::io::Error),
    #[error("no journal folder found; pass --journal-dir")]
    NoJournalDir,
    #[error("cannot read config: {0}")]
    ConfigRead(io::Error),
    #[error(transparent)]
    Config(#[from] bd_rules::ConfigError),
    #[error("cannot watch the journal folder: {0}")]
    Watch(String),
    #[error("could not connect to the board")]
    Connect,
}

/// Where the game's journal is, or `None` on an unsupported system.
fn detect_journal_dir() -> Option<PathBuf> {
    let platform = dir::current_platform()?;
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let xdg = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    let found = dir::candidates(platform, Path::new(&home), xdg.as_deref());
    dir::detect(&found, Path::is_dir)
}

/// Stands in for the board in `--dry-run`: swallows lines and always answers OK.
struct DryRun;

impl Transport for DryRun {
    fn read_line(&mut self, _timeout: Duration) -> io::Result<String> {
        Ok("OK".to_owned())
    }
    fn write_line(&mut self, _line: &str) -> io::Result<()> {
        Ok(())
    }
    fn discard_input(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Where the game data comes from.
enum Feed {
    Live(PathBuf),
    Replay(PathBuf),
}

/// Connect (failing fast on a wrong port), then run until the feed ends or Ctrl-C.
fn go<T: Transport, F: FnMut() -> io::Result<T>>(
    feed: Feed,
    config: &Config,
    mut link: Link<T, F>,
) -> Result<(), CliError> {
    if link.try_connect() != ConnectionState::Connected {
        return Err(CliError::Connect);
    }
    match feed {
        Feed::Replay(dir) => run::run_once(&ReplaySource::from_dir(&dir)?, config, &mut link)?,
        Feed::Live(dir) => {
            let (tx, rx) = mpsc::channel();
            // Kept alive for the whole loop; dropping it stops the notifications.
            let _watcher = watch(&dir, tx.clone()).map_err(|e| CliError::Watch(e.to_string()))?;
            let stop = Arc::new(AtomicBool::new(false));
            let flag = Arc::clone(&stop);
            // The extra wake-up ends the wait at once instead of after the poll interval.
            ctrlc::set_handler(move || {
                flag.store(true, Ordering::SeqCst);
                let _ = tx.send(());
            })
            .map_err(io::Error::other)?;
            run::run_loop(&DirSource(dir), config, &mut link, &rx, &stop)?;
        }
    }
    Ok(())
}

fn run_command(
    port: Option<String>,
    journal_dir: Option<PathBuf>,
    config: Option<PathBuf>,
    replay: Option<PathBuf>,
) -> Result<(), CliError> {
    let config = match config {
        Some(path) => {
            Config::from_toml(&std::fs::read_to_string(path).map_err(CliError::ConfigRead)?)?
        }
        None => Config::default(),
    };
    let feed = match replay {
        Some(dir) => Feed::Replay(dir),
        None => Feed::Live(
            journal_dir
                .or_else(detect_journal_dir)
                .ok_or(CliError::NoJournalDir)?,
        ),
    };
    match port {
        Some(port) => go(
            feed,
            &config,
            Link::new(move || SerialTransport::open(&port), REPLY_TIMEOUT),
        ),
        None => go(feed, &config, Link::new(|| Ok(DryRun), REPLY_TIMEOUT)),
    }
}

/// Words typed on the shell back into the one line the protocol parser expects.
fn parse_command(words: &[String]) -> Result<Command, ParseError> {
    words.join(" ").parse()
}

/// Check the command before touching the transport, so a typo never reaches the board.
fn send<T: Transport>(transport: T, words: &[String]) -> Result<Reply, CliError> {
    let command = parse_command(words)?;
    Ok(Device::new(transport, REPLY_TIMEOUT).send(&command)?)
}

fn run(cli: Cli) -> Result<ExitCode, CliError> {
    match cli.action {
        Action::Ports => {
            list_ports()?
                .iter()
                .for_each(|p| println!("{}\t{}", p.path, p.description));
            Ok(ExitCode::SUCCESS)
        }
        Action::Send { port, command } => {
            // Parse first so a bad command does not open (and reset) the board.
            parse_command(&command)?;
            let reply = send(SerialTransport::open(&port)?, &command)?;
            println!("{reply}");
            Ok(if reply == Reply::Ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
        Action::Run {
            port,
            journal_dir,
            config,
            dry_run: _,
            replay,
        } => {
            run_command(port, journal_dir, config, replay)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn main() -> ExitCode {
    run(Cli::parse()).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        ExitCode::FAILURE
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_device::fake::FakeDevice;
    use bd_protocol::{ErrCode, Mode};

    fn words(line: &str) -> Vec<String> {
        line.split(' ').map(str::to_owned).collect()
    }

    #[test]
    fn words_become_a_typed_command() {
        assert_eq!(
            parse_command(&words("mode active")),
            Ok(Command::Mode(Mode::Active))
        );
        assert!(matches!(
            parse_command(&words("6DIGIT SET 1000000")),
            Err(ParseError::Range(_))
        ));
        assert_eq!(parse_command(&words("NOPE")), Err(ParseError::Unknown));
    }

    #[test]
    fn send_prints_what_the_board_says() {
        let fake = FakeDevice::new();
        let reply = send(fake.clone(), &words("MODE ACTIVE")).ok();
        assert_eq!(reply, Some(Reply::Ok));
        let reply = send(fake.clone(), &words("LCD ROW 1 hi")).ok();
        assert_eq!(reply, Some(Reply::Ok));
        assert_eq!(fake.board().lcd[0], "hi");
    }

    #[test]
    fn send_to_a_demo_board_is_err_mode() {
        let reply = send(FakeDevice::new(), &words("LCD CLEAR")).ok();
        assert_eq!(reply, Some(Reply::Err(ErrCode::Mode)));
    }

    #[test]
    fn bad_command_never_reaches_the_board() {
        let fake = FakeDevice::new();
        assert!(matches!(
            send(fake.clone(), &words("LCD ROW 9 hi")),
            Err(CliError::Parse(_))
        ));
        assert_eq!(fake.board(), bd_device::fake::Board::new());
    }
}
