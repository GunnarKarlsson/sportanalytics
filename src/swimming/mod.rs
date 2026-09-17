//! Swimming analytics: Critical Swim Speed (CSS), training zones, race
//! prediction, World Aquatics points, and stroke-efficiency helpers.
//!
//! | Function | What it uses |
//! |---|---|
//! | [`css_from_trials`] / [`css_from_trials_n`] | Wakayoshi 1992 slope of D vs T (CSS + ADC) |
//! | [`css_from_t30`] | Distance covered in 1800 s |
//! | [`training_zones`] / [`training_zones_from_css`] | CSS pace ± coaching offsets per 100 m |
//! | [`predict_times`] / [`predict_from_css`] | CSS+ADC or Riegel `k = 1.03` |
//! | [`wa_points`] / [`wa_points_with`] | World Aquatics `P = floor(1000 (B/T)^3)` |
//! | [`swolf`] / [`distance_per_stroke`] | Length time + stroke count |
//!
//! Internal math stays in metres and seconds. Yards appear only at the I/O
//! edge ([`Event::from_yards`], [`Pace::per_100y`], [`LengthUnit::Per100y`]).
//!
//! CSS is effective threshold speed in water, not laboratory VO2max. Zone
//! edges are coaching offsets, not copyrighted printed CSS charts. World
//! Aquatics base times change by year — pass `B` via [`wa_points_with`] or a
//! [`WaPointsTable`].
//!
//! # Example
//!
//! ```
//! use sportanalytics::swimming::{
//!     css_from_trials, training_zones_from_css, Course, Event, Stroke, SwimTime,
//! };
//!
//! let t200 = SwimTime::from_hms_cents(
//!     Event::M200, Course::Scm, Stroke::Free, 0, 2, 30, 0,
//! ).unwrap();
//! let t400 = SwimTime::from_hms_cents(
//!     Event::M400, Course::Scm, Stroke::Free, 0, 5, 20, 0,
//! ).unwrap();
//! let css = css_from_trials(t200, t400).unwrap();
//! let zones = training_zones_from_css(css).unwrap();
//! assert!(zones.threshold.easy_end.sec_per_100m() > css.pace().unwrap().sec_per_100m());
//! ```
//!
//! Runnable example: `cargo run --example from_400_200` (source under
//! `examples/swimming/`).

mod course;
mod css;
mod distance;
mod efficiency;
mod points;
mod predict;
mod time;
mod units;
mod zones;

pub use course::{Course, Sex, Stroke};
pub use css::{css_from_1500, css_from_t30, css_from_trials, css_from_trials_n, Css};
pub use distance::Event;
pub use efficiency::{distance_per_stroke, swolf};
pub use points::{time_from_points_with, wa_points, wa_points_with, WaPointsTable};
pub use predict::{
    predict_from_css, predict_times, riegel, PredictedTimes, PredictionModel, RIEGEL_EXPONENT,
};
pub use time::SwimTime;
pub use units::{LengthUnit, Pace, PaceDisplay, METERS_PER_100Y, METERS_PER_YARD};
pub use zones::{training_zones, training_zones_from_css, PaceRange, TrainingZones};
