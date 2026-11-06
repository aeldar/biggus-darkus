//! `Status.json`: frequently changing cockpit info, replaced as a whole file.
//! <https://elite-journal.readthedocs.io/en/latest/Status%20File.html>

use bitflags::bitflags;
use serde::{Deserialize, Deserializer};

bitflags! {
    /// `Flags` bits. <https://elite-journal.readthedocs.io/en/latest/Status%20File.html>
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Flags: u32 {
        /// Docked, (on a landing pad)
        const DOCKED = 1 << 0;
        /// Landed, (on planet surface)
        const LANDED = 1 << 1;
        /// Landing gear down
        const LANDING_GEAR_DOWN = 1 << 2;
        /// Shields up
        const SHIELDS_UP = 1 << 3;
        /// Supercruise
        const SUPERCRUISE = 1 << 4;
        /// Flight assist off
        const FLIGHT_ASSIST_OFF = 1 << 5;
        /// Hardpoints deployed
        const HARDPOINTS_DEPLOYED = 1 << 6;
        /// In wing
        const IN_WING = 1 << 7;
        /// Lights on
        const LIGHTS_ON = 1 << 8;
        /// Cargo scoop deployed
        const CARGO_SCOOP_DEPLOYED = 1 << 9;
        /// Silent running
        const SILENT_RUNNING = 1 << 10;
        /// Scooping fuel
        const SCOOPING_FUEL = 1 << 11;
        /// SRV handbrake
        const SRV_HANDBRAKE = 1 << 12;
        /// SRV using turret view
        const SRV_TURRET_VIEW = 1 << 13;
        /// SRV turret retracted (close to ship)
        const SRV_TURRET_RETRACTED = 1 << 14;
        /// SRV drive assist
        const SRV_DRIVE_ASSIST = 1 << 15;
        /// FSD mass locked
        const FSD_MASS_LOCKED = 1 << 16;
        /// FSD charging
        const FSD_CHARGING = 1 << 17;
        /// FSD cooldown
        const FSD_COOLDOWN = 1 << 18;
        /// Low fuel (< 25%)
        const LOW_FUEL = 1 << 19;
        /// Over heating (> 100%)
        const OVER_HEATING = 1 << 20;
        /// Has latitude and longitude
        const HAS_LAT_LONG = 1 << 21;
        /// Is in danger
        const IS_IN_DANGER = 1 << 22;
        /// Being interdicted
        const BEING_INTERDICTED = 1 << 23;
        /// In main ship
        const IN_MAIN_SHIP = 1 << 24;
        /// In fighter
        const IN_FIGHTER = 1 << 25;
        /// In SRV
        const IN_SRV = 1 << 26;
        /// HUD in analysis mode
        const HUD_ANALYSIS_MODE = 1 << 27;
        /// Night vision
        const NIGHT_VISION = 1 << 28;
        /// Altitude is from average radius
        const ALTITUDE_FROM_AVERAGE_RADIUS = 1 << 29;
        /// FSD jump
        const FSD_JUMP = 1 << 30;
        /// SRV high beam
        const SRV_HIGH_BEAM = 1 << 31;
    }
}

bitflags! {
    /// `Flags2` bits. <https://elite-journal.readthedocs.io/en/latest/Status%20File.html>
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Flags2: u32 {
        /// On foot
        const ON_FOOT = 1 << 0;
        /// In taxi (or dropship/shuttle)
        const IN_TAXI = 1 << 1;
        /// In multicrew (in someone else's ship)
        const IN_MULTICREW = 1 << 2;
        /// On foot in station
        const ON_FOOT_IN_STATION = 1 << 3;
        /// On foot on planet
        const ON_FOOT_ON_PLANET = 1 << 4;
        /// Aim down sight
        const AIM_DOWN_SIGHT = 1 << 5;
        /// Low oxygen
        const LOW_OXYGEN = 1 << 6;
        /// Low health
        const LOW_HEALTH = 1 << 7;
        /// Cold
        const COLD = 1 << 8;
        /// Hot
        const HOT = 1 << 9;
        /// Very cold
        const VERY_COLD = 1 << 10;
        /// Very hot
        const VERY_HOT = 1 << 11;
        /// Glide mode
        const GLIDE_MODE = 1 << 12;
        /// On foot in hangar
        const ON_FOOT_IN_HANGAR = 1 << 13;
        /// On foot in social space
        const ON_FOOT_SOCIAL_SPACE = 1 << 14;
        /// On foot exterior
        const ON_FOOT_EXTERIOR = 1 << 15;
        /// Breathable atmosphere
        const BREATHABLE_ATMOSPHERE = 1 << 16;
        /// Telepresence multicrew
        const TELEPRESENCE_MULTICREW = 1 << 17;
        /// Physical multicrew
        const PHYSICAL_MULTICREW = 1 << 18;
        /// FSD hyperdrive charging
        const FSD_HYPERDRIVE_CHARGING = 1 << 19;
        /// Supercruise overdrive (SCO) active
        const SUPERCRUISE_OVERDRIVE = 1 << 20;
        /// Supercruise assist active
        const SUPERCRUISE_ASSIST = 1 << 21;
    }
}

// `from_bits_retain` keeps bits the docs do not name yet, so game updates do not lose data.
fn flags<'de, D: Deserializer<'de>>(d: D) -> Result<Flags, D::Error> {
    u32::deserialize(d).map(Flags::from_bits_retain)
}

fn flags2<'de, D: Deserializer<'de>>(d: D) -> Result<Flags2, D::Error> {
    u32::deserialize(d).map(Flags2::from_bits_retain)
}

/// Fuel tanks, both in tons.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Fuel {
    /// Main tank.
    pub fuel_main: f64,
    /// Reservoir (the small tank the FSD draws from).
    pub fuel_reservoir: f64,
}

/// `LegalState` values. Unlisted values map to `Other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum LegalState {
    /// No issues.
    Clean,
    /// Carrying illegal cargo.
    IllegalCargo,
    /// Speeding.
    Speeding,
    /// Wanted.
    Wanted,
    /// Hostile.
    Hostile,
    /// Carrying a wanted passenger.
    PassengerWanted,
    /// Warrant.
    Warrant,
    /// Allied.
    Allied,
    /// Thargoid.
    Thargoid,
    /// A value newer than the docs.
    #[serde(other)]
    Other,
}

/// The parts of `Status.json` we use.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    /// Ship/SRV/on-foot state bits.
    #[serde(default, deserialize_with = "flags")]
    pub flags: Flags,
    /// More bits, mainly for being on foot. Absent in older game versions.
    #[serde(default, deserialize_with = "flags2")]
    pub flags2: Flags2,
    /// Energy distribution (system, engine, weapons) in half-pips.
    pub pips: Option<[u8; 3]>,
    /// Selected fire group.
    pub fire_group: Option<u8>,
    /// Selected GUI screen (0 = no focus).
    pub gui_focus: Option<u8>,
    /// Fuel tanks.
    pub fuel: Option<Fuel>,
    /// Legal state.
    pub legal_state: Option<LegalState>,
}

/// Parse the contents of `Status.json`.
/// The game rewrites the file in place, so a read can catch it half written: callers retry on error.
pub fn parse_status(text: &str) -> Result<Status, serde_json::Error> {
    serde_json::from_str(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docs_example_docked_in_main_ship() {
        let s = parse_status(r#"{"timestamp":"2017-12-07T10:31:37Z","event":"Status","Flags":16842765,"Pips":[2,8,2],"FireGroup":0,"Fuel":{"FuelMain":15.146626,"FuelReservoir":0.382796},"GuiFocus":5}"#).expect("status");
        // The docs list this value as bits 24, 16, 3, 2, 0.
        assert_eq!(
            s.flags,
            Flags::IN_MAIN_SHIP
                | Flags::FSD_MASS_LOCKED
                | Flags::SHIELDS_UP
                | Flags::LANDING_GEAR_DOWN
                | Flags::DOCKED
        );
        assert_eq!(s.flags2, Flags2::empty());
        assert_eq!(s.pips, Some([2, 8, 2]));
        assert_eq!(s.gui_focus, Some(5));
        assert_eq!(s.fuel.map(|f| f.fuel_main), Some(15.146626));
    }

    #[test]
    fn docs_example_landed_with_lat_long() {
        let s = parse_status(r#"{"event":"Status","Flags":18874376,"Pips":[4,8,0],"FireGroup":0,"GuiFocus":0,"Latitude":-28.584963,"Longitude":6.826313,"Heading":109,"Altitude":404}"#).expect("status");
        assert!(s.flags.contains(Flags::HAS_LAT_LONG | Flags::SHIELDS_UP));
        assert!(!s.flags.contains(Flags::DOCKED));
    }

    #[test]
    fn bit_positions_match_the_docs() {
        assert_eq!(Flags::DOCKED.bits(), 1);
        assert_eq!(Flags::SUPERCRUISE.bits(), 16);
        assert_eq!(Flags::LOW_FUEL.bits(), 524_288);
        assert_eq!(Flags::FSD_JUMP.bits(), 1_073_741_824);
        assert_eq!(Flags::SRV_HIGH_BEAM.bits(), 2_147_483_648);
        assert_eq!(Flags2::ON_FOOT.bits(), 1);
        assert_eq!(Flags2::BREATHABLE_ATMOSPHERE.bits(), 65_536);
        assert_eq!(Flags2::SUPERCRUISE_ASSIST.bits(), 2_097_152);
    }

    #[test]
    fn flags2_and_unknown_bits_are_kept() {
        let s = parse_status(r#"{"Flags":0,"Flags2":2147483649}"#).expect("status");
        assert!(s.flags2.contains(Flags2::ON_FOOT));
        assert_eq!(s.flags2.bits(), 2_147_483_649);
    }

    #[test]
    fn missing_flags_default_to_empty() {
        let s = parse_status(r#"{"event":"Status"}"#).expect("status");
        assert_eq!(s.flags, Flags::empty());
        assert_eq!(s.fuel, None);
    }

    #[test]
    fn legal_state_with_unknown_value() {
        let s = parse_status(r#"{"LegalState":"Wanted"}"#).expect("status");
        assert_eq!(s.legal_state, Some(LegalState::Wanted));
        let s = parse_status(r#"{"LegalState":"Brand New"}"#).expect("status");
        assert_eq!(s.legal_state, Some(LegalState::Other));
    }

    #[test]
    fn half_written_file_is_an_error() {
        assert!(parse_status(r#"{"Flags":16842"#).is_err());
        assert!(parse_status("").is_err());
    }
}
