//! The resolver: which rule owns each output slot, and what changed since last time.
//!
//! An output slot is the smallest thing one rule can own:
//! one LED pixel, one LCD row, the 4-digit display, or the 6-digit display.
//! A rule claims every slot it covers, so rules on overlapping LED ranges compete
//! pixel by pixel: the winner of a pixel is the highest priority, ties go to the rule
//! listed first.

use std::collections::{BTreeMap, BTreeSet};

use bd_journal::{Event, GameState};
use bd_protocol::{
    Anim, Command, FourDigit, LED_COUNT, Lcd, LcdText, LedRange, LedStrip, Rgb, Row, SixDigit,
};

use crate::config::{Action, Config, EventKind, Rule, Trigger};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Slot {
    Led(u8),
    Lcd(u8),
    Four,
    Six,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Led(Rgb, Anim),
    Lcd(Row, Option<LcdText>),
    Four(FourDigit),
    Six(SixDigit),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Shown {
    value: Value,
    priority: i32,
    // Event output has no end condition, so it is carried over until something replaces it.
    latched: bool,
}

/// What the board shows now, per slot. Start from `Output::default()` (nothing shown).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output(BTreeMap<Slot, Shown>);

// The slots a rule covers, with the value it would put in each.
fn claims(action: &Action, state: &GameState) -> Vec<(Slot, Value)> {
    match action {
        Action::Led { range, color, anim } => (range.start()..range.end())
            .map(|n| (Slot::Led(n), Value::Led(*color, *anim)))
            .collect(),
        Action::Lcd { row, template } => vec![(
            Slot::Lcd(row.number()),
            Value::Lcd(*row, template.render(state)),
        )],
        Action::FourDigit(f) => vec![(Slot::Four, Value::Four(*f))],
        Action::SixDigit(s) => vec![(Slot::Six, Value::Six(*s))],
    }
}

fn active(rule: &Rule, state: &GameState, event: Option<&Event>) -> bool {
    match rule.trigger {
        Trigger::State(c) => c.holds(state),
        Trigger::Event(k) => event.and_then(EventKind::of) == Some(k),
    }
}

// A kept event value comes last, so on equal priority a fresh rule replaces it.
fn winners(
    config: &Config,
    state: &GameState,
    event: Option<&Event>,
    last: &Output,
) -> BTreeMap<Slot, Shown> {
    let fresh = config
        .rules
        .iter()
        .filter(|r| active(r, state, event))
        .flat_map(|r| {
            let latched = matches!(r.trigger, Trigger::Event(_));
            claims(&r.action, state)
                .into_iter()
                .map(move |(slot, value)| {
                    let priority = r.priority;
                    (
                        slot,
                        Shown {
                            value,
                            priority,
                            latched,
                        },
                    )
                })
        });
    let kept = last
        .0
        .iter()
        .filter(|(_, s)| s.latched)
        .map(|(slot, s)| (*slot, s.clone()));
    fresh
        .chain(kept)
        .fold(BTreeMap::new(), |mut acc, (slot, c)| {
            // Strictly greater, so the earlier candidate keeps a tie.
            let better = acc.get(&slot).is_none_or(|cur| c.priority > cur.priority);
            if better {
                acc.insert(slot, c);
            }
            acc
        })
}

// A blank LCD row is the same as no claim, so it never needs a command of its own.
fn visible(m: &BTreeMap<Slot, Shown>, slot: Slot) -> Option<&Value> {
    m.get(&slot)
        .map(|s| &s.value)
        .filter(|v| !matches!(v, Value::Lcd(_, None)))
}

fn led_at(m: &BTreeMap<Slot, Shown>, n: u8) -> Option<(Rgb, Anim)> {
    match visible(m, Slot::Led(n)) {
        Some(Value::Led(c, a)) => Some((*c, *a)),
        _ => None,
    }
}

// Merges neighbouring pixels with the same change into one command each.
fn led_commands(changes: Vec<(u8, Option<(Rgb, Anim)>)>) -> Vec<Command> {
    let runs = changes.into_iter().fold(
        Vec::<(u8, u8, Option<(Rgb, Anim)>)>::new(),
        |mut runs, (n, v)| {
            match runs.last_mut() {
                Some((_, end, last)) if *end == n && *last == v => *end = n + 1,
                _ => runs.push((n, n + 1, v)),
            }
            runs
        },
    );
    runs.into_iter()
        // Pixels are below LED_COUNT and runs are non-empty, so the range is always valid.
        .filter_map(|(s, e, v)| {
            let range = LedRange::new(s, e).ok()?;
            Some(Command::LedStrip(match v {
                Some((color, anim)) => LedStrip::Set { range, color, anim },
                None => LedStrip::ClearRange(range),
            }))
        })
        .collect()
}

// The 4-digit display has no blank, so clearing returns it to the clock; the 6-digit shows 0.
fn command_for(slot: Slot, new: Option<&Value>) -> Option<Command> {
    match (slot, new) {
        (_, Some(Value::Lcd(row, text))) => Some(Command::Lcd(Lcd::Row(*row, text.clone()))),
        (_, Some(Value::Four(f))) => Some(Command::FourDigit(*f)),
        (_, Some(Value::Six(s))) => Some(Command::SixDigit(*s)),
        (Slot::Lcd(n), _) => Row::new(u32::from(n))
            .ok()
            .map(|row| Command::Lcd(Lcd::Row(row, None))),
        (Slot::Four, _) => Some(Command::FourDigit(FourDigit::Time(None))),
        (Slot::Six, _) => Some(Command::SixDigit(SixDigit::Reset)),
        (Slot::Led(_), _) => None,
    }
}

fn diff(old: &BTreeMap<Slot, Shown>, new: &BTreeMap<Slot, Shown>) -> Vec<Command> {
    let leds = (0..LED_COUNT)
        .filter_map(|n| {
            let (o, w) = (led_at(old, n), led_at(new, n));
            (o != w).then_some((n, w))
        })
        .collect();
    let others: BTreeSet<Slot> = old
        .keys()
        .chain(new.keys())
        .copied()
        .filter(|s| !matches!(s, Slot::Led(_)))
        .collect();
    let rest = others
        .into_iter()
        .filter(|s| visible(old, *s) != visible(new, *s))
        .filter_map(|s| command_for(s, visible(new, s)));
    led_commands(leds).into_iter().chain(rest).collect()
}

/// The commands that move the board from `last` to what the rules say for `state`.
///
/// `state` is the state after `event` was applied. Pass the returned `Output` back in next
/// time; an unchanged state then yields no commands. Event rules stay on their slots until
/// a rule with the same or higher priority claims them (a state rule wins a tie).
/// Switching the board to `MODE ACTIVE` is the caller's job.
pub fn plan(
    config: &Config,
    state: &GameState,
    event: Option<&Event>,
    last: &Output,
) -> (Vec<Command>, Output) {
    let next = winners(config, state, event, last);
    (diff(&last.0, &next), Output(next))
}
