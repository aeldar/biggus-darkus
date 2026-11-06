//! What we know about the game right now, built by folding events and status snapshots.

use crate::event::{Event, JumpType};
use crate::status::Status;

/// Current game state. `Default` is "nothing seen yet".
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GameState {
    /// False after `Shutdown` until the next `LoadGame`.
    pub in_game: bool,
    /// False between `Died` and `Resurrect`/`LoadGame`.
    pub alive: bool,
    /// Commander name.
    pub commander: Option<String>,
    /// Ship type, as in `LoadGame`.
    pub ship: Option<String>,
    /// Current star system.
    pub system: Option<String>,
    /// Current body (star, planet or station name from the last event that gave one).
    pub body: Option<String>,
    /// Station we are docked at; `None` when not docked.
    pub station: Option<String>,
    /// Set from `StartJump` until the jump or supercruise transition ends.
    pub jumping: Option<JumpType>,
    /// Main tank, tons.
    pub fuel_main: Option<f64>,
    /// Reservoir, tons.
    pub fuel_reservoir: Option<f64>,
    /// Main tank size, tons.
    pub fuel_capacity: Option<f64>,
    /// Number of cargo units in the last full `Cargo` inventory.
    pub cargo_count: Option<u32>,
    /// Current music mood.
    pub music: Option<String>,
    /// Latest `Status.json` snapshot.
    pub status: Option<Status>,
}

/// Pure reducer: the next state after one journal event.
pub fn apply(state: GameState, event: Event) -> GameState {
    use Event::*;
    match event {
        LoadGame {
            commander,
            ship,
            fuel_level,
            fuel_capacity,
            ..
        } => GameState {
            in_game: true,
            alive: true,
            commander: Some(commander),
            ship,
            fuel_main: fuel_level.or(state.fuel_main),
            fuel_capacity: fuel_capacity.or(state.fuel_capacity),
            ..state
        },
        Cargo {
            inventory: Some(items),
            ..
        } => GameState {
            cargo_count: Some(items.iter().map(|i| i.count).sum()),
            ..state
        },
        Location {
            star_system,
            body,
            docked,
            station_name,
        } => GameState {
            system: Some(star_system),
            body,
            station: station_name.filter(|_| docked),
            ..state
        },
        FSDJump {
            star_system,
            body,
            fuel_level,
            ..
        } => GameState {
            system: Some(star_system),
            body,
            station: None,
            jumping: None,
            fuel_main: fuel_level.or(state.fuel_main),
            ..state
        },
        StartJump { jump_type, .. } => GameState {
            jumping: Some(jump_type),
            ..state
        },
        SupercruiseEntry { star_system } => GameState {
            jumping: None,
            system: star_system.or(state.system),
            ..state
        },
        SupercruiseExit { star_system, body } => GameState {
            jumping: None,
            system: star_system.or(state.system),
            body: body.or(state.body),
            ..state
        },
        Docked {
            station_name,
            star_system,
            ..
        } => GameState {
            station: Some(station_name.clone()),
            body: Some(station_name),
            system: star_system.or(state.system),
            ..state
        },
        Undocked { .. } => GameState {
            station: None,
            ..state
        },
        // Refuel events give only the amount bought, so cap by the tank size.
        RefuelAll { amount } | RefuelPartial { amount } => GameState {
            fuel_main: state.fuel_main.map(|f| {
                let sum = f + amount;
                state.fuel_capacity.map_or(sum, |cap| sum.min(cap))
            }),
            ..state
        },
        ReservoirReplenished {
            fuel_main,
            fuel_reservoir,
        } => GameState {
            fuel_main: Some(fuel_main),
            fuel_reservoir: Some(fuel_reservoir),
            ..state
        },
        FuelScoop { total, .. } => GameState {
            fuel_main: Some(total),
            ..state
        },
        Music { music_track } => GameState {
            music: Some(music_track),
            ..state
        },
        Died => GameState {
            alive: false,
            ..state
        },
        Resurrect => GameState {
            alive: true,
            ..state
        },
        Shutdown => GameState {
            in_game: false,
            ..state
        },
        // Touchdown/Liftoff/LaunchSRV show up in Status flags; the rest we do not track.
        Cargo { .. } | Touchdown { .. } | Liftoff { .. } | LaunchSRV | Unknown => state,
    }
}

/// Fold a `Status.json` snapshot into state. Its fuel is fresher than the journal's.
pub fn apply_status(state: GameState, status: Status) -> GameState {
    GameState {
        fuel_main: status.fuel.map(|f| f.fuel_main).or(state.fuel_main),
        fuel_reservoir: status
            .fuel
            .map(|f| f.fuel_reservoir)
            .or(state.fuel_reservoir),
        status: Some(status),
        ..state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{CargoItem, parse_line};
    use crate::status::{Flags, parse_status};

    fn fold(lines: &[&str]) -> GameState {
        lines
            .iter()
            .map(|l| parse_line(l).expect("event"))
            .fold(GameState::default(), apply)
    }

    const LOAD: &str = r#"{"event":"LoadGame","Commander":"HRC-2","Ship":"FerDeLance","FuelLevel":4.0,"FuelCapacity":8}"#;

    #[test]
    fn load_game_sets_basics() {
        let s = fold(&[LOAD]);
        assert!(s.in_game && s.alive);
        assert_eq!(s.commander.as_deref(), Some("HRC-2"));
        assert_eq!(s.ship.as_deref(), Some("FerDeLance"));
        assert_eq!(s.fuel_main, Some(4.0));
        assert_eq!(s.fuel_capacity, Some(8.0));
    }

    #[test]
    fn location_docked_and_not_docked() {
        let s = fold(&[
            r#"{"event":"Location","StarSystem":"Luhman 16","Docked":true,"StationName":"Jenner Orbital"}"#,
        ]);
        assert_eq!(s.station.as_deref(), Some("Jenner Orbital"));
        let s = fold(&[r#"{"event":"Location","StarSystem":"Eranin","Body":"Eranin 1"}"#]);
        assert_eq!(s.station, None);
        assert_eq!(s.body.as_deref(), Some("Eranin 1"));
    }

    #[test]
    fn dock_undock_cycle() {
        let s = fold(&[
            r#"{"event":"Docked","StationName":"Jenner Orbital","StarSystem":"Luhman 16"}"#,
        ]);
        assert_eq!(s.station.as_deref(), Some("Jenner Orbital"));
        assert_eq!(s.system.as_deref(), Some("Luhman 16"));
        let s = apply(s, Event::Undocked { station_name: None });
        assert_eq!(s.station, None);
        assert_eq!(s.system.as_deref(), Some("Luhman 16"));
    }

    #[test]
    fn jump_sequence() {
        let s = fold(&[
            LOAD,
            r#"{"event":"StartJump","JumpType":"Hyperspace","StarSystem":"Eranin"}"#,
        ]);
        assert_eq!(s.jumping, Some(JumpType::Hyperspace));
        let s = apply(
            s,
            parse_line(
                r#"{"event":"FSDJump","StarSystem":"Eranin","Body":"Eranin","FuelLevel":3.5}"#,
            )
            .expect("event"),
        );
        assert_eq!(s.jumping, None);
        assert_eq!(s.system.as_deref(), Some("Eranin"));
        assert_eq!(s.fuel_main, Some(3.5));
        assert_eq!(s.station, None);
    }

    #[test]
    fn supercruise_transitions() {
        let s = fold(&[
            r#"{"event":"StartJump","JumpType":"Supercruise"}"#,
            r#"{"event":"SupercruiseEntry","StarSystem":"Yuetu"}"#,
        ]);
        assert_eq!(s.jumping, None);
        assert_eq!(s.system.as_deref(), Some("Yuetu"));
        let s = apply(
            s,
            parse_line(r#"{"event":"SupercruiseExit","StarSystem":"Yuetu","Body":"Yuetu 2"}"#)
                .expect("event"),
        );
        assert_eq!(s.body.as_deref(), Some("Yuetu 2"));
    }

    #[test]
    fn refuel_is_capped_at_capacity() {
        let s = fold(&[LOAD, r#"{"event":"RefuelPartial","Amount":1.5}"#]);
        assert_eq!(s.fuel_main, Some(5.5));
        let s = apply(s, Event::RefuelAll { amount: 99.0 });
        assert_eq!(s.fuel_main, Some(8.0));
    }

    #[test]
    fn refuel_without_known_fuel_stays_unknown() {
        let s = fold(&[r#"{"event":"RefuelAll","Amount":2.0}"#]);
        assert_eq!(s.fuel_main, None);
    }

    #[test]
    fn scoop_and_reservoir_set_fuel() {
        let s = fold(&[LOAD, r#"{"event":"FuelScoop","Scooped":0.5,"Total":6.0}"#]);
        assert_eq!(s.fuel_main, Some(6.0));
        let s = apply(
            s,
            Event::ReservoirReplenished {
                fuel_main: 5.9,
                fuel_reservoir: 0.4,
            },
        );
        assert_eq!((s.fuel_main, s.fuel_reservoir), (Some(5.9), Some(0.4)));
    }

    #[test]
    fn cargo_counts_only_full_inventories() {
        let s = apply(
            GameState::default(),
            Event::Cargo {
                vessel: Some("Ship".into()),
                inventory: Some(vec![
                    CargoItem {
                        name: "gold".into(),
                        count: 2,
                    },
                    CargoItem {
                        name: "tea".into(),
                        count: 5,
                    },
                ]),
            },
        );
        assert_eq!(s.cargo_count, Some(7));
        let s = fold_on(s, r#"{"event":"Cargo","Vessel":"Ship"}"#);
        assert_eq!(s.cargo_count, Some(7));
    }

    fn fold_on(s: GameState, line: &str) -> GameState {
        apply(s, parse_line(line).expect("event"))
    }

    #[test]
    fn death_and_shutdown() {
        let s = fold(&[LOAD, r#"{"event":"Died"}"#]);
        assert!(!s.alive);
        let s = fold_on(s, r#"{"event":"Resurrect","Option":"rebuy"}"#);
        assert!(s.alive);
        let s = fold_on(s, r#"{"event":"Shutdown"}"#);
        assert!(!s.in_game);
        let s = fold_on(s, LOAD);
        assert!(s.in_game);
    }

    #[test]
    fn music_and_unknown() {
        let s = fold(&[r#"{"event":"Music","MusicTrack":"Combat_Dogfight"}"#]);
        assert_eq!(s.music.as_deref(), Some("Combat_Dogfight"));
        let before = s.clone();
        assert_eq!(fold_on(s, r#"{"event":"Scan"}"#), before);
    }

    #[test]
    fn status_overrides_fuel_and_is_kept() {
        let s = fold(&[LOAD]);
        let st = parse_status(r#"{"Flags":16,"Fuel":{"FuelMain":2.5,"FuelReservoir":0.3}}"#)
            .expect("status");
        let s = apply_status(s, st);
        assert_eq!((s.fuel_main, s.fuel_reservoir), (Some(2.5), Some(0.3)));
        assert_eq!(s.status.map(|x| x.flags), Some(Flags::SUPERCRUISE));
    }

    #[test]
    fn status_without_fuel_keeps_journal_fuel() {
        let s = fold(&[LOAD]);
        let s = apply_status(s, parse_status(r#"{"Flags":0}"#).expect("status"));
        assert_eq!(s.fuel_main, Some(4.0));
    }
}
