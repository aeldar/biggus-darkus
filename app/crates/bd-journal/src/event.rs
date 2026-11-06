//! Journal events. Only the ones a companion device uses are modelled; everything
//! else parses as `Unknown`, so new game updates never break us.
//! File layout and event list: <https://elite-journal.readthedocs.io/en/latest/File%20Format.html>
// Field names mirror the official docs one to one, so per-field docs would only repeat them.
#![allow(missing_docs)]

use serde::Deserialize;

/// One cargo line of a `Cargo` event.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CargoItem {
    pub name: String,
    pub count: u32,
}

/// Why a jump countdown started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum JumpType {
    Hyperspace,
    Supercruise,
}

/// A journal line, tagged by its `"event"` key.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "event")]
pub enum Event {
    /// <https://elite-journal.readthedocs.io/en/latest/Startup.html> (LoadGame)
    #[serde(rename_all = "PascalCase")]
    LoadGame {
        commander: String,
        ship: Option<String>,
        ship_name: Option<String>,
        fuel_level: Option<f64>,
        fuel_capacity: Option<f64>,
    },
    /// <https://elite-journal.readthedocs.io/en/latest/Startup.html> (Cargo)
    /// Later `Cargo` events carry no inventory: it moved to Cargo.json.
    #[serde(rename_all = "PascalCase")]
    Cargo {
        vessel: Option<String>,
        inventory: Option<Vec<CargoItem>>,
    },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (Location)
    #[serde(rename_all = "PascalCase")]
    Location {
        star_system: String,
        body: Option<String>,
        #[serde(default)]
        docked: bool,
        station_name: Option<String>,
    },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (FSDJump)
    #[serde(rename_all = "PascalCase")]
    FSDJump {
        star_system: String,
        body: Option<String>,
        jump_dist: Option<f64>,
        fuel_used: Option<f64>,
        fuel_level: Option<f64>,
    },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (StartJump)
    #[serde(rename_all = "PascalCase")]
    StartJump {
        jump_type: JumpType,
        star_system: Option<String>,
    },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (SupercruiseEntry)
    #[serde(rename_all = "PascalCase")]
    SupercruiseEntry { star_system: Option<String> },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (SupercruiseExit)
    #[serde(rename_all = "PascalCase")]
    SupercruiseExit {
        star_system: Option<String>,
        body: Option<String>,
    },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (Docked)
    #[serde(rename_all = "PascalCase")]
    Docked {
        station_name: String,
        station_type: Option<String>,
        star_system: Option<String>,
    },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (Undocked)
    #[serde(rename_all = "PascalCase")]
    Undocked { station_name: Option<String> },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (Touchdown)
    #[serde(rename_all = "PascalCase")]
    Touchdown { player_controlled: Option<bool> },
    /// <https://elite-journal.readthedocs.io/en/latest/Travel.html> (Liftoff)
    #[serde(rename_all = "PascalCase")]
    Liftoff { player_controlled: Option<bool> },
    /// <https://elite-journal.readthedocs.io/en/latest/Station%20Services.html> (RefuelAll)
    #[serde(rename_all = "PascalCase")]
    RefuelAll { amount: f64 },
    /// <https://elite-journal.readthedocs.io/en/latest/Station%20Services.html> (RefuelPartial)
    #[serde(rename_all = "PascalCase")]
    RefuelPartial { amount: f64 },
    /// <https://elite-journal.readthedocs.io/en/latest/Other%20Events.html> (ReservoirReplenished)
    #[serde(rename_all = "PascalCase")]
    ReservoirReplenished { fuel_main: f64, fuel_reservoir: f64 },
    /// <https://elite-journal.readthedocs.io/en/latest/Other%20Events.html> (FuelScoop)
    #[serde(rename_all = "PascalCase")]
    FuelScoop { scooped: f64, total: f64 },
    /// <https://elite-journal.readthedocs.io/en/latest/Other%20Events.html> (LaunchSRV)
    LaunchSRV,
    /// <https://elite-journal.readthedocs.io/en/latest/Other%20Events.html> (Music)
    #[serde(rename_all = "PascalCase")]
    Music { music_track: String },
    /// <https://elite-journal.readthedocs.io/en/latest/Combat.html> (Died)
    Died,
    /// <https://elite-journal.readthedocs.io/en/latest/Other%20Events.html> (Resurrect)
    Resurrect,
    /// <https://elite-journal.readthedocs.io/en/latest/Other%20Events.html> (Shutdown)
    Shutdown,
    /// Any event we do not model (including the `fileheader` line).
    #[serde(other)]
    Unknown,
}

/// Parse one journal line.
pub fn parse_line(line: &str) -> Result<Event, serde_json::Error> {
    serde_json::from_str(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(json: &str) -> Event {
        parse_line(json).expect("valid event")
    }

    #[test]
    fn load_game_from_docs_example() {
        let e = ev(
            r#"{"timestamp":"2017-02-10T14:25:51Z","event":"LoadGame","Commander":"HRC-2","FID":"F44396","Horizons":true,"Ship":"FerDeLance","ShipID":19,"ShipName":"jewel of parhoon","ShipIdent":"hr-17f","FuelLevel":3.964024,"FuelCapacity":8,"GameMode":"Open","Credits":2890718739,"Loan":0}"#,
        );
        assert_eq!(
            e,
            Event::LoadGame {
                commander: "HRC-2".into(),
                ship: Some("FerDeLance".into()),
                ship_name: Some("jewel of parhoon".into()),
                fuel_level: Some(3.964024),
                fuel_capacity: Some(8.0),
            }
        );
    }

    #[test]
    fn cargo_with_and_without_inventory() {
        let full = ev(
            r#"{"event":"Cargo","Vessel":"Ship","Inventory":[{"Name":"gold","Count":2,"Stolen":0},{"Name":"iondistributor","Name_Localised":"Ion Distributor","Count":2,"Stolen":0}]}"#,
        );
        match full {
            Event::Cargo {
                inventory: Some(items),
                ..
            } => assert_eq!(items.iter().map(|i| i.count).sum::<u32>(), 4),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(
            ev(r#"{"event":"Cargo","Vessel":"Ship"}"#),
            Event::Cargo {
                vessel: Some("Ship".into()),
                inventory: None
            }
        );
    }

    #[test]
    fn location_defaults_docked_to_false() {
        let e = ev(r#"{"event":"Location","StarSystem":"Eranin","Body":"Eranin A"}"#);
        assert_eq!(
            e,
            Event::Location {
                star_system: "Eranin".into(),
                body: Some("Eranin A".into()),
                docked: false,
                station_name: None
            }
        );
    }

    #[test]
    fn fsd_jump_ignores_extra_fields() {
        let e = ev(
            r#"{"event":"FSDJump","StarSystem":"Eranin","StarPos":[-22.8,36.5,-1.1],"JumpDist":13.334,"FuelUsed":0,"FuelLevel":25.630281,"Factions":[{"Name":"x"}]}"#,
        );
        assert_eq!(
            e,
            Event::FSDJump {
                star_system: "Eranin".into(),
                body: None,
                jump_dist: Some(13.334),
                fuel_used: Some(0.0),
                fuel_level: Some(25.630281)
            }
        );
    }

    #[test]
    fn jump_types() {
        assert_eq!(
            ev(r#"{"event":"StartJump","JumpType":"Supercruise"}"#),
            Event::StartJump {
                jump_type: JumpType::Supercruise,
                star_system: None
            }
        );
    }

    #[test]
    fn unit_events_accept_extra_fields() {
        assert_eq!(
            ev(
                r#"{"event":"Died","KillerName":"$ShipName_Police_Independent;","KillerShip":"viper","KillerRank":"Deadly"}"#
            ),
            Event::Died
        );
        assert_eq!(ev(r#"{"event":"Shutdown"}"#), Event::Shutdown);
    }

    #[test]
    fn fuel_events() {
        assert_eq!(
            ev(r#"{"event":"FuelScoop","Scooped":0.4987,"Total":16}"#),
            Event::FuelScoop {
                scooped: 0.4987,
                total: 16.0
            }
        );
        assert_eq!(
            ev(r#"{"event":"RefuelAll","Cost":317,"Amount":6.322901}"#),
            Event::RefuelAll { amount: 6.322901 }
        );
        assert_eq!(
            ev(r#"{"event":"ReservoirReplenished","FuelMain":15.1,"FuelReservoir":0.38}"#),
            Event::ReservoirReplenished {
                fuel_main: 15.1,
                fuel_reservoir: 0.38
            }
        );
    }

    #[test]
    fn music_track() {
        assert_eq!(
            ev(r#"{"event":"Music","MusicTrack":"Supercruise"}"#),
            Event::Music {
                music_track: "Supercruise".into()
            }
        );
    }

    #[test]
    fn unknown_events_fall_back() {
        assert_eq!(ev(r#"{"event":"Scan","BodyName":"x"}"#), Event::Unknown);
        assert_eq!(
            ev(r#"{"timestamp":"2016-07-22T10:20:01Z","event":"fileheader","part":1}"#),
            Event::Unknown
        );
    }

    #[test]
    fn malformed_line_is_an_error() {
        assert!(parse_line(r#"{"event":"Loc"#).is_err());
        assert!(parse_line(r#"{"no":"tag"}"#).is_err());
    }

    #[test]
    fn known_event_missing_required_field_is_an_error() {
        assert!(parse_line(r#"{"event":"Docked"}"#).is_err());
    }
}
