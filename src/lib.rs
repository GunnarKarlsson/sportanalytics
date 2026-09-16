//! Sport analytics crate. Sports live in their own modules so more can be added later.
//!
//! Crate name: `sportanalytics`. Repository: `sports-analytics`.
//!
//! Start with [`running`]. See `examples/from_5k.rs` and `examples/age_grade.rs`.
//!
//! Formulas: Daniels & Gilbert (1979) VDOT, Riegel (1977/1981), and Cameron’s
//! road-race fit. Training paces are inverted from those equations, not copied
//! from copyrighted Daniels pace tables. Published *Running Formula* charts
//! will differ by a few seconds/km. Age factors are a WMA-style
//! *approximation*, not official scoring tables.
//!
//! VDOT is *effective* VO2max (running economy included), not a laboratory test.
//!
//! Enable the `serde` feature to serialize public types. Default builds stay
//! dependency-free.
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
//! let _easy = sportanalytics::running::format_pace(zones.easy.hard_end);
//!
//! let ag = age_grade(five, 42, Gender::Male, Some(25));
//! assert!(ag.percent > 50.0);
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(rust_2018_idioms, missing_debug_implementations)]

mod error;
pub mod running;

pub use error::Error;

/// Common running types and functions.
///
/// Prefer [`crate::running`] when adding another sport so names stay scoped.
pub mod prelude {
    pub use crate::running::{
        age_equivalent, age_grade, predict_daniels_and_cameron, predict_times, training_zones,
        training_zones_from_vdot, vdot, vo2max_from_races, AgeGradeResult, Distance,
        DualPredictedTimes, Gender, PaceRange, PerformanceLevel, PredictedTimes, PredictionModel,
        RaceTime, TrainingZones, Vdot, Vo2Estimate,
    };
    pub use crate::Error;
}
