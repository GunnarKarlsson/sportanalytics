//! Sport analytics crate. Sports live in their own modules so more can be added later.
//!
//! Crate name: `sportanalytics`. Repository: `sportanalytics`.
//!
//! Version **0.2** ships [`running`] and [`cycling`]. See
//! `examples/running/from_5k.rs`, `examples/running/age_grade.rs`,
//! `examples/cycling/from_20min.rs`, and `examples/cycling/from_tt.rs`.
//!
//! **Running:** Daniels & Gilbert, *Oxygen Power* (1979) VDOT equations (not
//! copyrighted printed pace grids), Riegel (1977/1981, `k = 1.06`), and
//! Cameron’s road-race fit. Training paces are inverted from those equations.
//! Age grading uses the official USATF MLDR 2025 road tables.
//!
//! **Cycling:** operational FTP protocols, two-parameter critical power, Coggan
//! %FTP zones, ACSM relative VO2 (plus Hawley–Noakes absolute), Martin 1998
//! power–speed, and a trained-endurance age-factor curve (not official
//! age-grade tables).
//!
//! Enable the `serde` feature to serialize public types. Default builds stay
//! dependency-free. New sports are **not** re-exported from the crate root.
//!
//! Sports return [`running::Error`] and [`cycling::Error`]. Mixed-sport binaries
//! can use `Box<dyn std::error::Error>`.
//!
//! # Example
//!
//! ```
//! use sportanalytics::running::{
//!     age_grade, predict_daniels_and_cameron, training_zones, vo2max_from_races, Distance,
//!     Gender, RaceTime,
//! };
//!
//! let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
//! let ten = RaceTime::from_hms(Distance::TenK, 0, 42, 0).unwrap();
//!
//! let vo2 = vo2max_from_races(&[five, ten]).unwrap();
//! assert!(vo2.best.value() > 45.0);
//!
//! let both = predict_daniels_and_cameron(five).unwrap();
//! let _hm = both.daniels.formatted(Distance::HalfMarathon).unwrap();
//!
//! let zones = training_zones(five);
//! let _easy = zones.easy.hard_end.to_string();
//!
//! let ag = age_grade(five, 42, Gender::Male, Some(25)).unwrap();
//! assert!(ag.percent > 50.0);
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(rust_2018_idioms, missing_debug_implementations)]

pub mod cycling;
pub mod running;

/// Common running types and functions.
///
/// Prefer [`crate::running`] when adding another sport so names stay scoped.
/// Cycling lives under [`crate::cycling`] (not this prelude).
///
/// Pace and zone [`std::fmt::Display`] default to `/km`. For miles, call
/// [`.display(LengthUnit::Mile)`](crate::running::Pace::display) (also on
/// [`PaceRange`](crate::running::PaceRange) and
/// [`TrainingZones`](crate::running::TrainingZones)).
pub mod prelude {
    pub use crate::running::{
        age_equivalent, age_grade, predict_daniels_and_cameron, predict_times, training_zones,
        training_zones_from_vdot, vdot, vo2max_from_races, AgeGradeResult, AgeGradeTable, Distance,
        DualPredictedTimes, Gender, LengthUnit, Pace, PaceRange, PerformanceLevel, PredictedTimes,
        PredictionModel, RaceTime, TrainingZones, Vdot, Vo2Estimate, METERS_PER_MILE,
    };
}
