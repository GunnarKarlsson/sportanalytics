//! Running analytics: VDOT / effective VO2max, race-time prediction, Daniels
//! training zones, and a compact WMA-style age-grade model.
//!
//! | Function | What it uses |
//! |---|---|
//! | [`vdot`] / [`vo2max_from_races`] | Daniels–Gilbert 1979 VDOT (effective VO2max) |
//! | [`predict_times`] | Daniels invert, Riegel `T2=T1*(D2/D1)^1.06`, or Cameron |
//! | [`predict_daniels_and_cameron`] | Daniels and Cameron in one call |
//! | [`training_zones`] / [`training_zones_from_vdot`] | Daniels %VDOT pace bands (E/M/T/I/R) |
//! | [`age_grade()`] / [`age_equivalent()`] | Compact WMA-style age factors + open standards |
//!
//! Age and gender are used only by age grading. Daniels, Riegel, and Cameron
//! predictions do not take them. Predict first, then pass a predicted time into
//! [`age_equivalent`] if you need an age-adjusted figure.
//!
//! # Types
//!
//! - [`Distance`] — `ThreeK`, `FiveK`, `TenK`, `HalfMarathon`, `Marathon`, or
//!   [`Distance::from_meters`] / [`Distance::custom`]. Half marathon is 21,097.5 m;
//!   marathon is 42,195 m.
//! - [`RaceTime`] — a distance plus a positive finish time (`from_hms`,
//!   `from_secs`, or `new`). Minutes and seconds must be `< 60`.
//! - [`Vdot`] — newtype around a positive finite Daniels VDOT. Cameron/Riegel
//!   times are *not* VDOT values.
//! - [`crate::Error`] — `NonPositiveTime`, `EmptyRaces`, `InvalidVdot`,
//!   `InvalidDistance`, `UnrecognizedDistance`, `InvalidHms`, `InvalidPace`,
//!   `UnsolvableTime`.
//!
//! # Training zones
//!
//! | Zone | % of VDOT | Use |
//! |------|-----------|-----|
//! | Easy (E) | 59–74% | easy / long run |
//! | Marathon (M) | 75–84% | marathon pace |
//! | Threshold (T) | 83–88% | tempo / cruise intervals |
//! | Interval (I) | 95–100% | 3–5 min VO2 reps |
//! | Repetition (R) | ~105–110% | short fast reps |
//!
//! Paces are inverted from the oxygen-cost equations, not copied from Daniels’
//! published (copyrighted) charts. Published *Running Formula* charts will
//! differ by a few seconds/km.
//!
//! # Age grading
//!
//! [`Gender`] selects WMA male/female table standards, not a general gender
//! model. Open 5K–marathon times are 2025-era road world records (USATF MLDR
//! 2025 open standards); age-grade % can run a few points high versus 2015/2020
//! championship tables. 3K is a track-adjacent stand-in.
//!
//! Performance bands: ≥100% world-record level, ≥90% world class, ≥80% national,
//! ≥70% regional, ≥60% local, ≥50% recreational, else developing.
//!
//! # Examples
//!
//! `cargo run --example from_5k` (VDOT, predictions, zones) and
//! `cargo run --example age_grade` (42-year-old 5K).

mod age_grade;
mod distance;
mod predict;
mod time;
mod units;
mod vo2;
mod zones;

pub use age_grade::{
    age_equivalent, age_factor, age_grade, open_standard_secs, AgeGradeResult, Gender,
    PerformanceLevel,
};
pub use distance::Distance;
pub use predict::{
    cameron, predict_daniels_and_cameron, predict_times, riegel, riegel_with_exponent,
    DualPredictedTimes, PredictedTimes, PredictionModel,
};
pub use time::RaceTime;
pub use units::{LengthUnit, Pace, PaceDisplay, METERS_PER_MILE};
pub use vo2::{
    oxygen_cost, percent_vo2max, time_from_vdot, vdot, velocity_from_vo2, vo2max_from_races, Vdot,
    Vo2Estimate,
};
pub use zones::{format_pace, training_zones, training_zones_from_vdot, PaceRange, TrainingZones};
