//! Headless runner and debug tool.

use std::{process::ExitCode, time::Duration};

use bd_device::{Device, SerialTransport, Transport, list_ports};
use bd_protocol::{Command, ParseError, Reply};
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
}

#[derive(Debug, Error)]
enum CliError {
    #[error("invalid command: {0}")]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Device(#[from] bd_device::Error),
    #[error("cannot open port: {0}")]
    Port(#[from] std::io::Error),
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
