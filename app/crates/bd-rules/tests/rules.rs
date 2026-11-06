//! Table tests: a sequence of (state, event) in, the exact wire commands out.
#![allow(clippy::expect_used, clippy::panic)]

use bd_journal::{
    Event, GameState,
    event::parse_line,
    status::{Flags, Status, parse_status},
};
use bd_rules::{Config, ConfigError, Output, RuleError, plan};

fn status(flags: Flags) -> Status {
    parse_status(&format!(r#"{{"Flags":{}}}"#, flags.bits())).expect("status")
}

fn state(flags: Flags) -> GameState {
    GameState {
        in_game: true,
        status: Some(status(flags)),
        ..GameState::default()
    }
}

fn event(line: &str) -> Event {
    parse_line(line).expect("event")
}

type Step = (GameState, Option<Event>, Vec<&'static str>);

// Feeds each step the previous output, like the real loop, and compares wire text.
fn run(config: &Config, steps: Vec<Step>) {
    steps
        .into_iter()
        .enumerate()
        .fold(Output::default(), |last, (i, (st, ev, want))| {
            let (cmds, next) = plan(config, &st, ev.as_ref(), &last);
            let got: Vec<String> = cmds.iter().map(ToString::to_string).collect();
            assert_eq!(got, want, "step {i}");
            next
        });
}

fn cfg(rules: &str) -> Config {
    Config::from_toml(&format!("version = 1\n{rules}")).expect("config")
}

#[test]
fn overlapping_leds_resolve_per_pixel() {
    let c = cfg(r#"
[[rules]]
priority = 10
trigger = { state = "gear_down" }
action = { output = "led", start = 0, end = 4, color = "00FF00" }
[[rules]]
priority = 20
trigger = { state = "hardpoints_deployed" }
action = { output = "led", start = 2, end = 6, color = "FF0000", anim = "blink" }
"#);
    let gear = Flags::LANDING_GEAR_DOWN;
    let hard = Flags::HARDPOINTS_DEPLOYED;
    run(
        &c,
        vec![
            (state(Flags::empty()), None, vec![]),
            (
                state(gear),
                None,
                vec!["LED_STRIP RANGE 0 4 COLOR 00FF00 ANIM SOLID"],
            ),
            // Repeated identical state sends nothing.
            (state(gear), None, vec![]),
            // Pixels 2..4 go from green to red; 4..6 are new and the same: one command.
            (
                state(gear | hard),
                None,
                vec!["LED_STRIP RANGE 2 6 COLOR FF0000 ANIM BLINK"],
            ),
            // Gear up: the lower rule's pixels 0..2 clear, the winner keeps 2..6.
            (state(hard), None, vec!["LED_STRIP RANGE 0 2 CLEAR"]),
            // Hardpoints away: the lower rule shows again where it still holds.
            (
                state(gear),
                None,
                vec![
                    "LED_STRIP RANGE 0 4 COLOR 00FF00 ANIM SOLID",
                    "LED_STRIP RANGE 4 6 CLEAR",
                ],
            ),
            (
                state(Flags::empty()),
                None,
                vec!["LED_STRIP RANGE 0 4 CLEAR"],
            ),
        ],
    );
}

#[test]
fn equal_priority_goes_to_the_first_rule() {
    let c = cfg(r#"
[[rules]]
trigger = { state = "docked" }
action = { output = "led", start = 0, end = 2, color = "0000FF" }
[[rules]]
trigger = { state = "gear_down" }
action = { output = "led", start = 0, end = 2, color = "00FF00" }
"#);
    run(
        &c,
        vec![(
            state(Flags::DOCKED | Flags::LANDING_GEAR_DOWN),
            None,
            vec!["LED_STRIP RANGE 0 2 COLOR 0000FF ANIM SOLID"],
        )],
    );
}

#[test]
fn lcd_rows_truncate_compete_and_clear() {
    let c = cfg(r#"
[[rules]]
trigger = { state = "in_game" }
action = { output = "lcd", row = 1, text = "{system}" }
[[rules]]
priority = 10
trigger = { state = "docked" }
action = { output = "lcd", row = 1, text = "DOCKED {station}" }
"#);
    let at = |s: &str| GameState {
        system: Some(s.into()),
        station: None,
        ..state(Flags::empty())
    };
    let docked = GameState {
        station: Some("Jenner Orbital".into()),
        ..at("Luhman 16")
    };
    run(
        &c,
        vec![
            (at("Luhman 16"), None, vec!["LCD ROW 1 Luhman 16"]),
            (at("Luhman 16"), None, vec![]),
            // 16 characters is the row width.
            (
                at("A Very Long System Name"),
                None,
                vec!["LCD ROW 1 A Very Long Syst"],
            ),
            (docked, None, vec!["LCD ROW 1 DOCKED Jenner Or"]),
            (at("Luhman 16"), None, vec!["LCD ROW 1 Luhman 16"]),
            // Condition over and nothing below it: the row clears.
            (GameState::default(), None, vec!["LCD ROW 1"]),
        ],
    );
}

#[test]
fn unknown_value_renders_blank_row() {
    let c = cfg(r#"
[[rules]]
trigger = { state = "in_game" }
action = { output = "lcd", row = 2, text = "{station}" }
"#);
    // Nothing to show and nothing shown before: no command.
    run(&c, vec![(state(Flags::empty()), None, vec![])]);
}

#[test]
fn event_output_stays_until_a_state_rule_with_equal_priority_takes_over() {
    let c = cfg(r#"
[[rules]]
priority = 5
trigger = { event = "FSDJump" }
action = { output = "lcd", row = 2, text = "Jumped" }
[[rules]]
priority = 1
trigger = { state = "in_game" }
action = { output = "lcd", row = 2, text = "{body}" }
[[rules]]
priority = 5
trigger = { state = "supercruise" }
action = { output = "lcd", row = 2, text = "Cruising" }
"#);
    let jump = event(r#"{"event":"FSDJump","StarSystem":"Eranin"}"#);
    let base = GameState {
        body: Some("Eranin 1".into()),
        ..state(Flags::empty())
    };
    let cruise = GameState {
        body: Some("Eranin 1".into()),
        ..state(Flags::SUPERCRUISE)
    };
    run(
        &c,
        vec![
            (base.clone(), None, vec!["LCD ROW 2 Eranin 1"]),
            (base.clone(), Some(jump), vec!["LCD ROW 2 Jumped"]),
            // No event now, but a lower state rule must not erase it.
            (base.clone(), None, vec![]),
            (cruise, None, vec!["LCD ROW 2 Cruising"]),
            (base, None, vec!["LCD ROW 2 Eranin 1"]),
        ],
    );
}

#[test]
fn digit_displays_set_and_clear() {
    let c = cfg(r#"
[[rules]]
trigger = { state = "docked" }
action = { output = "four_digit", show = "timer", seconds = 300 }
[[rules]]
trigger = { state = "docked" }
action = { output = "six_digit", show = "set", value = 42 }
"#);
    run(
        &c,
        vec![
            (state(Flags::empty()), None, vec![]),
            (
                state(Flags::DOCKED),
                None,
                vec!["4DIGIT TIMER 300", "6DIGIT SET 42"],
            ),
            (state(Flags::DOCKED), None, vec![]),
            // The 4-digit display has no blank, so it goes back to the clock.
            (
                state(Flags::empty()),
                None,
                vec!["4DIGIT TIME", "6DIGIT RESET"],
            ),
        ],
    );
}

#[test]
fn state_conditions() {
    let c = cfg(r#"
[[rules]]
trigger = { state = "shields_down" }
action = { output = "led", start = 0, end = 1, color = "FF0000" }
[[rules]]
trigger = { state = "low_fuel" }
action = { output = "led", start = 1, end = 2, color = "FFB000", anim = "pulse" }
[[rules]]
trigger = { state = "fsd_charging" }
action = { output = "led", start = 2, end = 3, color = "0000FF" }
"#);
    let low_by_numbers = GameState {
        fuel_main: Some(1.0),
        fuel_capacity: Some(8.0),
        ..state(Flags::SHIELDS_UP)
    };
    run(
        &c,
        vec![
            // No Status.json yet: shields are unknown, not down.
            (GameState::default(), None, vec![]),
            (state(Flags::SHIELDS_UP), None, vec![]),
            (
                state(Flags::empty()),
                None,
                vec!["LED_STRIP RANGE 0 1 COLOR FF0000 ANIM SOLID"],
            ),
            (
                low_by_numbers,
                None,
                vec![
                    "LED_STRIP RANGE 0 1 CLEAR",
                    "LED_STRIP RANGE 1 2 COLOR FFB000 ANIM PULSE",
                ],
            ),
            (
                state(Flags::SHIELDS_UP | Flags::FSD_CHARGING),
                None,
                vec![
                    "LED_STRIP RANGE 1 2 CLEAR",
                    "LED_STRIP RANGE 2 3 COLOR 0000FF ANIM SOLID",
                ],
            ),
        ],
    );
}

#[test]
fn default_rules_work_out_of_the_box() {
    let c = Config::default();
    assert!(!c.rules.is_empty(), "default.toml must load");
    let st = GameState {
        system: Some("Luhman 16".into()),
        body: Some("Luhman 16 A".into()),
        ..state(Flags::SHIELDS_UP | Flags::LANDING_GEAR_DOWN | Flags::LOW_FUEL)
    };
    run(
        &c,
        vec![(
            st,
            None,
            vec![
                "LED_STRIP RANGE 0 6 COLOR 00FF00 ANIM SOLID",
                "LED_STRIP RANGE 34 46 COLOR FFB000 ANIM PULSE 600",
                "LCD ROW 1 Luhman 16",
                "LCD ROW 2 Luhman 16 A",
            ],
        )],
    );
}

#[test]
fn default_toml_file_is_the_default() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/default.toml"))
        .expect("default.toml");
    assert_eq!(Config::from_toml(&text).expect("parse"), Config::default());
}

fn rule_with(action: &str) -> String {
    format!("version = 1\n[[rules]]\ntrigger = {{ state = \"docked\" }}\naction = {action}\n")
}

#[test]
fn config_errors_are_clear() {
    // (config text, text the error must contain)
    let cases: Vec<(String, &str)> = vec![
        ("version = 2".into(), "unsupported config version 2"),
        ("rules = []".into(), "version"),
        (rule_with(r#"{ output = "led", start = 5, end = 5, color = "FF0000" }"#), "rule 1: LED range out of range"),
        (rule_with(r#"{ output = "led", start = 0, end = 47, color = "FF0000" }"#), "LED range out of range"),
        (rule_with(r#"{ output = "led", start = 0, end = 999, color = "FF0000" }"#), "LED range out of range"),
        (rule_with(r#"{ output = "led", start = 0, end = 4, color = "red" }"#), "invalid colour `red`"),
        (rule_with(r##"{ output = "led", start = 0, end = 4, color = "#FF0000" }"##), "invalid colour"),
        (rule_with(r#"{ output = "led", start = 0, end = 4, color = "FF0000", period_ms = 100 }"#), "no `period_ms`"),
        (rule_with(r#"{ output = "led", start = 0, end = 4, color = "FF0000", anim = "pulse", period_ms = 0 }"#), "period out of range"),
        (rule_with(r#"{ output = "lcd", row = 3, text = "x" }"#), "LCD row out of range"),
        (rule_with(r#"{ output = "lcd", row = 1, text = "12345678901234567" }"#), "LCD text out of range"),
        (rule_with(r#"{ output = "lcd", row = 1, text = "" }"#), "LCD text out of range"),
        (rule_with(r#"{ output = "lcd", row = 1, text = "{nope}" }"#), "unknown field `{nope}`"),
        (rule_with(r#"{ output = "lcd", row = 1, text = "{system" }"#), "unclosed"),
        (rule_with(r#"{ output = "four_digit", show = "time", at = "25:00" }"#), "clock time out of range"),
        (rule_with(r#"{ output = "four_digit", show = "time", at = "noon" }"#), "invalid time `noon`"),
        (rule_with(r#"{ output = "four_digit", show = "timer", at = "10:00" }"#), "4-digit"),
        (rule_with(r#"{ output = "four_digit", show = "timer", seconds = 400000 }"#), "seconds out of range"),
        (rule_with(r#"{ output = "six_digit", show = "set", value = 1000000 }"#), "counter out of range"),
        (rule_with(r#"{ output = "six_digit", show = "count_up" }"#), "needs `value`"),
        (rule_with(r#"{ output = "lcd", row = 1, text = "x", colour = "typo" }"#), "unknown field"),
        ("version = 1\n[[rules]]\ntrigger = { state = \"gear_up\" }\naction = { output = \"six_digit\", show = \"reset\" }".into(), "unknown variant"),
    ];
    cases.iter().for_each(|(text, want)| {
        let err = Config::from_toml(text).expect_err(text);
        assert!(err.to_string().contains(want), "{text}\n  got: {err}");
    });
}

#[test]
fn error_variants_are_typed() {
    let err = Config::from_toml(&rule_with(
        r#"{ output = "led", start = 0, end = 4, color = "zz" }"#,
    ))
    .expect_err("bad colour");
    assert!(matches!(
        err,
        ConfigError::Rule {
            number: 1,
            source: RuleError::Color(_)
        }
    ));
}

#[test]
fn valid_four_and_six_digit_forms_load() {
    for action in [
        r#"{ output = "four_digit", show = "time" }"#,
        r#"{ output = "four_digit", show = "time", at = "07:05" }"#,
        r#"{ output = "four_digit", show = "startwatch", pause = true }"#,
        r#"{ output = "six_digit", show = "reset" }"#,
        r#"{ output = "six_digit", show = "count_down", value = 100, period_ms = 50 }"#,
    ] {
        Config::from_toml(&rule_with(action)).expect(action);
    }
}
