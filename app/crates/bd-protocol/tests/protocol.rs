//! Round-trip, example and golden-file tests.

use std::error::Error;

use bd_protocol::*;
use proptest::prelude::*;

type TestResult = Result<(), Box<dyn Error>>;

fn counter() -> impl Strategy<Value = Counter> {
    (0..=COUNTER_MAX).prop_filter_map("in range", |v| Counter::new(v).ok())
}

fn period() -> impl Strategy<Value = PeriodMs> {
    (1..=u32::from(u16::MAX)).prop_filter_map("in range", |v| PeriodMs::new(v).ok())
}

fn seconds() -> impl Strategy<Value = Seconds> {
    (0..=SECONDS_MAX).prop_filter_map("in range", |v| Seconds::new(v).ok())
}

fn range() -> impl Strategy<Value = LedRange> {
    (0..LED_COUNT, 1..=LED_COUNT).prop_filter_map("start < end", |(s, e)| LedRange::new(s, e).ok())
}

fn watch() -> impl Strategy<Value = Watch> {
    prop_oneof![
        Just(Watch::Resume),
        Just(Watch::Pause),
        seconds().prop_map(Watch::Start)
    ]
}

fn anim() -> impl Strategy<Value = Anim> {
    prop_oneof![
        Just(Anim::Solid),
        proptest::option::of(period()).prop_map(Anim::Blink),
        proptest::option::of(period()).prop_map(Anim::Pulse),
    ]
}

fn command() -> impl Strategy<Value = Command> {
    let mode =
        prop_oneof![Just(Mode::Demo), Just(Mode::Active), Just(Mode::Off)].prop_map(Command::Mode);
    let time = proptest::option::of(
        (0u8..24, 0u8..60).prop_filter_map("valid", |(h, m)| ClockTime::new(h, m).ok()),
    )
    .prop_map(|t| Command::FourDigit(FourDigit::Time(t)));
    let four = prop_oneof![
        watch().prop_map(|w| Command::FourDigit(FourDigit::Startwatch(w))),
        watch().prop_map(|w| Command::FourDigit(FourDigit::Timer(w))),
    ];
    let opt_period = || proptest::option::of(period());
    let six = prop_oneof![
        counter().prop_map(|v| Command::SixDigit(SixDigit::Set(v))),
        Just(Command::SixDigit(SixDigit::Reset)),
        (counter(), opt_period())
            .prop_map(|(from, period)| Command::SixDigit(SixDigit::CountUp { from, period })),
        (counter(), opt_period())
            .prop_map(|(from, period)| Command::SixDigit(SixDigit::CountDown { from, period })),
    ];
    // Spaces are allowed anywhere in LCD text, including the ends.
    let text =
        proptest::option::of("[ -~]{1,16}".prop_filter_map("valid", |t| LcdText::new(&t).ok()));
    let lcd = prop_oneof![
        (prop_oneof![Just(Row::One), Just(Row::Two)], text)
            .prop_map(|(r, t)| Command::Lcd(Lcd::Row(r, t))),
        Just(Command::Lcd(Lcd::Clear)),
    ];
    let led = prop_oneof![
        (range(), any::<[u8; 3]>(), anim()).prop_map(|(range, [r, g, b], anim)| Command::LedStrip(
            LedStrip::Set {
                range,
                color: Rgb { r, g, b },
                anim
            }
        )),
        range().prop_map(|r| Command::LedStrip(LedStrip::ClearRange(r))),
        Just(Command::LedStrip(LedStrip::Clear)),
    ];
    prop_oneof![mode, time, four, six, lcd, led]
}

proptest! {
    #[test]
    fn command_round_trips(cmd in command()) {
        let line = cmd.encode().map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(line.as_str().parse::<Command>(), Ok(cmd));
    }

    #[test]
    fn reply_round_trips(code in prop::sample::select(ErrCode::ALL.to_vec())) {
        for r in [Reply::Ok, Reply::Err(code)] {
            prop_assert_eq!(r.to_string().parse::<Reply>(), Ok(r));
        }
    }
}

#[test]
fn spec_examples_re_encode_to_themselves() -> TestResult {
    for kind in CommandKind::ALL {
        let example = spec(kind).example;
        let cmd: Command = example.parse()?;
        assert_eq!(cmd.kind(), kind, "{example}");
        assert_eq!(cmd.encode()?.as_str(), example);
    }
    Ok(())
}

#[test]
fn firmware_examples_parse_and_re_encode() -> TestResult {
    let lines = include_str!("firmware_commands.txt")
        .lines()
        .filter(|l| !l.starts_with('#'));
    let mut count = 0;
    for line in lines {
        let cmd: Command = line.parse()?;
        // Firmware examples are already canonical, so the text must survive unchanged.
        assert_eq!(cmd.encode()?.as_str(), line);
        count += 1;
    }
    assert!(count >= 20, "fixture looks empty: {count} lines");
    Ok(())
}

#[test]
fn keywords_ignore_case_but_text_keeps_it() -> TestResult {
    let lower: Command = "led_strip range 0 23 color ff0000 anim blink 500".parse()?;
    assert_eq!(
        lower.encode()?.as_str(),
        "LED_STRIP RANGE 0 23 COLOR FF0000 ANIM BLINK 500"
    );
    let lcd: Command = "lcd row 1 Hello World".parse()?;
    assert_eq!(lcd.encode()?.as_str(), "LCD ROW 1 Hello World");
    Ok(())
}

#[test]
fn bad_lines_map_to_the_firmware_errors() {
    let err = |s: &str| s.parse::<Command>().err();
    assert_eq!(err("FOO 1"), Some(ParseError::Unknown));
    assert_eq!(err("MODE"), Some(ParseError::Args));
    assert_eq!(
        err("LED_STRIP RANGE 0 47 CLEAR"),
        Some(OutOfRange("LED range").into())
    );
    assert_eq!(
        err("LED_STRIP RANGE 5 5 CLEAR"),
        Some(OutOfRange("LED range").into())
    );
    assert_eq!(
        err("6DIGIT SET 1000000"),
        Some(OutOfRange("counter").into())
    );
    assert_eq!(err("LCD ROW 3 x"), Some(ParseError::Args));
    assert_eq!(
        err("LCD ROW 1 12345678901234567"),
        Some(OutOfRange("LCD text").into())
    );
    assert_eq!(err(&"A".repeat(65)), Some(ParseError::Line));
}

#[test]
fn replies_parse() -> TestResult {
    assert_eq!("OK\r\n".parse::<Reply>()?, Reply::Ok);
    assert_eq!("err range".parse::<Reply>()?, Reply::Err(ErrCode::Range));
    assert!("ERR NOPE".parse::<Reply>().is_err());
    Ok(())
}

#[test]
fn checked_in_docs_are_current() {
    assert_eq!(
        include_str!("../../../docs/protocol.md"),
        render_markdown(),
        "docs/protocol.md is stale; run `cargo run -p bd-protocol --bin gen-docs`"
    );
}
