//! Running analytics: VDOT / effective VO2max, race-time prediction, Daniels
//! training zones, and USATF MLDR 2025 road age grading.
//!
//! | Function | What it uses |
//! |---|---|
//! | [`vdot`] / [`vo2max_from_races`] | Daniels–Gilbert 1979 VDOT (effective VO2max) |
//! | [`predict_times`] | Daniels invert, Riegel `T2=T1*(D2/D1)^1.06`, or Cameron |
//! | [`predict_daniels_and_cameron`] | Daniels and Cameron in one call |
//! | [`training_zones`] / [`training_zones_from_vdot`] | Daniels %VDOT pace bands (E/M/T/I/R) |
//! | [`age_grade()`] / [`age_equivalent()`] | USATF MLDR 2025 single-year road tables |
//!
//! Age and gender are used only by age grading. Daniels, Riegel, and Cameron
//! predictions do not take them. Predict first, then pass a predicted time into
//! [`age_equivalent`] if you need an age-adjusted figure.
//!
//! # Types
//!
//! - [`Distance`] — `ThreeK`, `FiveK`, `TenK`, `HalfMarathon`, `Marathon`, or
//!   [`Distance::from_meters`] / [`Distance::from_km`] / [`Distance::from_miles`] /
//!   [`Distance::custom`]. Half marathon is 21,097.5 m; marathon is 42,195 m.
//!   Strings such as `"8mi"` parse via [`str::parse`].
//! - [`RaceTime`] — a distance plus a positive finish time (`from_hms`,
//!   `from_secs`, `from_pace`, or `new`). Minutes and seconds must be `< 60`.
//! - [`Pace`] — seconds per metre internally. Default [`std::fmt::Display`] is
//!   `m:ss /km`; use [`Pace::display`] with [`LengthUnit::Mile`] for `/mi`.
//! - [`LengthUnit`] — kilometre (default) or international mile
//!   ([`METERS_PER_MILE`] m) at the I/O edge only.
//! - [`Vdot`] — newtype around a positive finite Daniels VDOT. Cameron/Riegel
//!   times are *not* VDOT values.
//! - [`Error`] — every failure running constructors and helpers can return
//!   (`NonPositiveTime`, `InvalidHms`, `AgeOutOfRange { min, max }`, and model
//!   variants).
//!
//! Internal math stays in metres and seconds. Kilometre vs mile appears only
//! when constructing or displaying distances and paces.
//!
//! # Training zones
//!
//! | Zone | % of VDOT | Use |
//! |------|-----------|-----|
//! | Easy (E) | 0.59–0.74 | easy / long run |
//! | Marathon (M) | 0.75–0.84 | marathon pace |
//! | Threshold (T) | 0.83–0.88 | tempo / cruise intervals |
//! | Interval (I) | 0.95–1.00 | 3–5 min VO2 reps |
//! | Repetition (R) | 1.05–1.10 | short fast reps |
//!
//! Paces are inverted from the oxygen-cost equations, not copied from Daniels’
//! published (copyrighted) charts. Published *Running Formula* charts will
//! differ by a few seconds/km. Zone edges are [`Pace`] values; default display
//! is `/km`, or [`TrainingZones::display`] / [`PaceRange::display`] with
//! [`LengthUnit::Mile`] for `/mi`.
//!
//! # Age grading
//!
//! [`age_grade()`], [`age_factor()`], [`age_equivalent()`], and [`open_standard_secs()`]
//! return [`Result`](Error). Ages are **5..=99** ([`Error::AgeOutOfRange`]
//! `{ min: 5, max: 99 }` otherwise). Road 3K is unsupported (`UnsupportedAgeGradeDistance`). Off-grid
//! distances interpolate age standards in log-distance between neighbouring
//! official events. [`Gender`] selects the male/female table columns.
//! [`PerformanceLevel`] bands are informal community labels, not official WMA
//! awards. Tables are the official USATF MLDR 2025 road tables (Alan Jones /
//! Tom Bernhard, approved 2025-01-10, CC0).
//!
//! # Examples
//!
//! `cargo run --example from_5k` (VDOT, predictions, zones) and
//! `cargo run --example age_grade` (42-year-old 5K). Sources live under
//! `examples/running/`.

mod age_grade;
mod distance;
mod error;
mod predict;
mod time;
mod units;
mod vo2;
mod zones;

pub use age_grade::{
    age_equivalent, age_equivalent_with, age_factor, age_factor_with, age_grade, age_grade_with,
    open_standard_secs, open_standard_secs_with, AgeGradeResult, AgeGradeTable, Gender,
    PerformanceLevel,
};
pub use distance::Distance;
pub use error::Error;
pub use predict::{
    cameron, predict_daniels_and_cameron, predict_times, riegel, riegel_with_exponent,
    DualPredictedTimes, PredictedTimes, PredictionModel, RIEGEL_EXPONENT,
};
pub use time::RaceTime;
pub use units::{LengthUnit, Pace, PaceDisplay, METERS_PER_MILE};
pub use vo2::{
    oxygen_cost, percent_vo2max, time_from_vdot, vdot, velocity_from_vo2, vo2max_from_races, Vdot,
    Vo2Estimate,
};
#[allow(deprecated)]
pub use zones::format_pace;
pub use zones::{training_zones, training_zones_from_vdot, PaceRange, TrainingZones};
