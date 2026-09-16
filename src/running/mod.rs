//! Running analytics: VDOT/VO2max, age-graded %, equivalent age times,
//! race predictions, and Daniels training zones.

mod age_grade;
mod distance;
mod predict;
mod time;
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
pub use vo2::{
    oxygen_cost, percent_vo2max, time_from_vdot, vdot, velocity_from_vo2, vo2max_from_races, Vdot,
    Vo2Estimate,
};
pub use zones::{format_pace, training_zones, training_zones_from_vdot, PaceRange, TrainingZones};
