//! Cycling analytics: FTP, critical power, Coggan zones, power–duration
//! prediction, Hawley–Noakes VO2, Martin road power–speed, and an age-factor
//! helper.
//!
//! | Function | What it uses |
//! |---|---|
//! | [`ftp_from_20min`] / [`ftp_from_protocol`] | Operational FTP from field protocols |
//! | [`critical_power()`] | Two-parameter CP + W′ (OLS on `1/t`) |
//! | [`predict_power`] / [`predict_from_cp`] | CP, %FTP curve, or power Riegel |
//! | [`training_zones`] | Coggan Z1–Z7 + sweet spot (%FTP) |
//! | [`estimated_vo2max`] | Hawley & Noakes MAP → VO2 field estimate |
//! | [`power_for_speed`] / [`speed_for_power`] | Martin et al. 1998 power balance |
//! | [`age_equivalent_ftp`] | Trained-endurance decline curve (not official tables) |
//!
//! Internal units are watts, seconds, metres, and kilograms. FTP is an
//! operational training anchor, not laboratory lactate threshold. Estimated
//! VO2 is a field estimate, not gas analysis. Age helpers use ages **15..=90**
//! and are **not** USATF/VTTA age grading.
//!
//! # Example
//!
//! ```
//! use sportanalytics::cycling::{ftp_from_20min, training_zones, Effort};
//!
//! let twenty = Effort::from_watts_secs(280.0, 20.0 * 60.0).unwrap();
//! let ftp = ftp_from_20min(twenty).unwrap();
//! assert!((ftp.watts() - 266.0).abs() < 1e-9);
//! let zones = training_zones(ftp);
//! assert!(zones.endurance.hard_end.watts() > zones.endurance.easy_end.watts());
//! ```
//!
//! Runnable examples: `cargo run --example from_20min` and
//! `cargo run --example from_tt` (sources under `examples/cycling/`).

mod age;
mod critical_power;
mod effort;
mod ftp;
mod physics;
mod predict;
mod vo2;
mod zones;

pub use age::{age_equivalent_ftp, age_factor, AEROBIC_DECLINE_PER_YEAR, AGE_PLATEAU};
pub use critical_power::{
    critical_power, predict_duration_from_cp, predict_power_from_cp, CpFit, CriticalPower,
};
pub use effort::{watts_per_kg, Effort, Mass, Power, WattsPerKg, Work};
pub use ftp::{
    ftp_from_20min, ftp_from_60min, ftp_from_cp, ftp_from_map, ftp_from_protocol, Ftp, FtpProtocol,
    CP_TO_FTP, EIGHT_MIN_FACTOR, MAP_TO_FTP, TWENTY_MIN_FACTOR,
};
pub use physics::{
    climb_time, power_for_speed, speed_for_power, time_for_distance, vam_m_per_hour, Environment,
    RiderBike, DEFAULT_CDA, DEFAULT_CRR, DEFAULT_ETA, G, SEA_LEVEL_RHO,
};
pub use predict::{
    predict_duration, predict_from_cp, predict_power, predict_power_from_efforts, riegel_power,
    PredictionModel, RIEGEL_POWER_EXPONENT,
};
pub use vo2::{
    estimated_vo2max, estimated_vo2max_from_ftp, map_from_vo2, EstimatedVo2, FTP_TO_MAP,
    HAWLEY_NOAKES_A, HAWLEY_NOAKES_B,
};
pub use zones::{training_zones, training_zones_from_effort, PowerRange, PowerZones};
