//! `Device` and `Link` against the fake firmware.
// Tests may unwrap: a failure is the report.
#![allow(clippy::expect_used)]

use std::time::Duration;

use bd_device::{
    ConnectionState, Device, Error, Link, Transport,
    fake::{FakeDevice, Led},
};
use bd_protocol::{Anim, Command, ErrCode, Mode, Reply, Rgb};

const TIMEOUT: Duration = Duration::from_millis(50);

fn cmd(line: &str) -> Command {
    line.parse().expect("test command parses")
}

fn connected() -> (FakeDevice, Device<FakeDevice>) {
    let fake = FakeDevice::new();
    let device = Device::connect(fake.clone(), TIMEOUT).expect("fake accepts MODE ACTIVE");
    (fake, device)
}

#[test]
fn connect_activates_and_shutdown_returns_to_demo() {
    let (fake, mut device) = connected();
    assert_eq!(fake.board().mode, Mode::Active);
    device.shutdown().expect("shutdown");
    assert_eq!(fake.board().mode, Mode::Demo);
}

#[test]
fn commands_change_the_board() {
    let (fake, mut device) = connected();
    for line in [
        "LCD ROW 1 Biggus",
        "LCD ROW 2 Darkus",
        "LCD ROW 2",
        "LED_STRIP RANGE 0 23 COLOR FF0000 ANIM SOLID",
        "LED_STRIP RANGE 20 25 COLOR 0000FF ANIM PULSE 1000",
        "LED_STRIP RANGE 0 2 CLEAR",
    ] {
        assert_eq!(device.send(&cmd(line)).expect(line), Reply::Ok);
    }
    let board = fake.board();
    assert_eq!(board.lcd, ["Biggus".to_owned(), String::new()]);
    assert_eq!(board.leds[0], None);
    let red = Some(Led {
        color: Rgb { r: 255, g: 0, b: 0 },
        anim: Anim::Solid,
    });
    assert_eq!(board.leds[2], red);
    assert_eq!(board.leds[19], red);
    assert_eq!(board.leds[20].map(|l| l.color.b), Some(255));
    assert_eq!(board.leds[24].map(|l| l.color.b), Some(255));
    assert_eq!(board.leds[25], None);
}

#[test]
fn command_before_active_is_err_mode_and_changes_nothing() {
    let fake = FakeDevice::new();
    let mut device = Device::new(fake.clone(), TIMEOUT);
    let reply = device.send(&cmd("LCD ROW 1 hi")).expect("reply");
    assert_eq!(reply, Reply::Err(ErrCode::Mode));
    assert_eq!(fake.board().lcd, <[String; 2]>::default());
}

#[test]
fn command_after_shutdown_is_err_mode() {
    let (_fake, mut device) = connected();
    device.shutdown().expect("shutdown");
    let reply = device.send(&cmd("LED_STRIP CLEAR")).expect("reply");
    assert_eq!(reply, Reply::Err(ErrCode::Mode));
}

#[test]
fn silent_board_times_out_and_recovers() {
    let (fake, mut device) = connected();
    fake.set_silent(true);
    assert!(matches!(
        device.send(&cmd("LCD CLEAR")),
        Err(Error::Timeout)
    ));
    fake.set_silent(false);
    assert_eq!(device.send(&cmd("LCD CLEAR")).expect("reply"), Reply::Ok);
}

#[test]
fn connect_to_silent_board_fails() {
    let fake = FakeDevice::new();
    fake.set_silent(true);
    assert!(matches!(
        Device::connect(fake, TIMEOUT),
        Err(Error::Timeout)
    ));
}

#[test]
fn unplugged_port_is_an_io_error() {
    let (fake, mut device) = connected();
    fake.set_unplugged(true);
    assert!(matches!(device.send(&cmd("LCD CLEAR")), Err(Error::Io(_))));
}

#[test]
fn link_reconnects_after_unplug() {
    let fake = FakeDevice::new();
    let source = fake.clone();
    let open = move || Ok(source.clone());
    let mut link = Link::new(open, TIMEOUT);

    assert!(matches!(
        link.send(&cmd("LCD CLEAR")),
        Err(Error::NotConnected)
    ));
    assert_eq!(link.try_connect(), ConnectionState::Connected);
    assert_eq!(link.send(&cmd("LCD CLEAR")).expect("reply"), Reply::Ok);

    fake.set_unplugged(true);
    assert!(matches!(link.send(&cmd("LCD CLEAR")), Err(Error::Io(_))));
    assert_eq!(link.state(), ConnectionState::Reconnecting { attempt: 0 });

    // Still unplugged: opening works but MODE ACTIVE fails.
    assert_eq!(
        link.try_connect(),
        ConnectionState::Reconnecting { attempt: 1 }
    );

    fake.set_unplugged(false);
    assert_eq!(link.try_connect(), ConnectionState::Connected);
    assert_eq!(link.send(&cmd("LCD CLEAR")).expect("reply"), Reply::Ok);
}

/// A board that ignores its first `skip` lines, like a Nano still booting.
struct Booting {
    board: FakeDevice,
    skip: u32,
}

impl Transport for Booting {
    fn read_line(&mut self, timeout: Duration) -> std::io::Result<String> {
        self.board.read_line(timeout)
    }

    fn write_line(&mut self, line: &str) -> std::io::Result<()> {
        match self.skip.checked_sub(1) {
            Some(left) => {
                self.skip = left;
                Ok(())
            }
            None => self.board.write_line(line),
        }
    }

    fn discard_input(&mut self) -> std::io::Result<()> {
        self.board.discard_input()
    }
}

#[test]
fn connect_waits_for_a_booting_board() {
    let transport = Booting {
        board: FakeDevice::new(),
        skip: 3,
    };
    assert!(Device::connect(transport, TIMEOUT).is_ok());
}

#[test]
fn connect_gives_up_on_a_board_that_never_answers() {
    let transport = Booting {
        board: FakeDevice::new(),
        skip: 100,
    };
    assert!(matches!(
        Device::connect(transport, TIMEOUT),
        Err(Error::Timeout)
    ));
}

#[test]
fn link_keeps_the_reason_a_connect_failed() {
    let fake = FakeDevice::new();
    fake.set_silent(true);
    let mut link = Link::new(move || Ok(fake.clone()), TIMEOUT);
    link.try_connect();
    assert_eq!(link.last_error(), Some("no reply from the board"));
}
