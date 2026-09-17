//! Training zones from Critical Swim Speed.
//!
//! | Zone | vs CSS pace (s / 100 m) |
//! |------|-------------------------|
//! | Recovery | +15 … +25 |
//! | Aerobic | +8 … +15 |
//! | Steady | +4 … +8 |
//! | Threshold | −2 … +2 |
//! | Vo2 | −8 … −3 |
//! | Sprint | −15 … −8 |

use std::fmt;

use super::css::{css_from_trials, Css};
use super::units::{LengthUnit, Pace};
use super::SwimTime;
use crate::Error;

// Zone edges are coaching offsets in seconds per 100 m from CSS pace.
// They are not copied from any copyrighted printed CSS chart.

/// A pace band with a slower and a faster edge.
///
/// `easy_end` is the slower edge; `hard_end` is the faster edge.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PaceRange {
    /// Slower edge.
    pub easy_end: Pace,
    /// Faster edge.
    pub hard_end: Pace,
}

impl PaceRange {
    /// Format both edges in `unit` (`/100m` or `/100y`).
    pub const fn display(self, unit: LengthUnit) -> PaceRangeDisplay {
        PaceRangeDisplay { range: self, unit }
    }
}

impl fmt::Display for PaceRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}–{}", self.easy_end, self.hard_end)
    }
}

/// A [`PaceRange`] paired with a [`LengthUnit`] for formatting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaceRangeDisplay {
    range: PaceRange,
    unit: LengthUnit,
}

impl fmt::Display for PaceRangeDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}–{}",
            self.range.easy_end.display(self.unit),
            self.range.hard_end.display(self.unit)
        )
    }
}

/// CSS-based training zones (coaching offsets per 100 m).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrainingZones {
    /// CSS used to compute the zones.
    pub css: Css,
    /// Recovery: CSS pace +15 … +25 s/100m.
    pub recovery: PaceRange,
    /// Aerobic: CSS pace +8 … +15 s/100m.
    pub aerobic: PaceRange,
    /// Steady: CSS pace +4 … +8 s/100m.
    pub steady: PaceRange,
    /// Threshold: CSS pace −2 … +2 s/100m.
    pub threshold: PaceRange,
    /// VO2: CSS pace −8 … −3 s/100m.
    pub vo2: PaceRange,
    /// Sprint: CSS pace −15 … −8 s/100m.
    pub sprint: PaceRange,
}

impl TrainingZones {
    /// Format all zones in `unit` (`/100m` or `/100y`).
    pub const fn display(self, unit: LengthUnit) -> TrainingZonesDisplay {
        TrainingZonesDisplay { zones: self, unit }
    }
}

/// A [`TrainingZones`] value paired with a [`LengthUnit`] for formatting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrainingZonesDisplay {
    zones: TrainingZones,
    unit: LengthUnit,
}

impl fmt::Display for TrainingZones {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rec {}  Aer {}  Std {}  Thr {}  VO2 {}  Spr {}",
            self.recovery, self.aerobic, self.steady, self.threshold, self.vo2, self.sprint
        )
    }
}

impl fmt::Display for TrainingZonesDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rec {}  Aer {}  Std {}  Thr {}  VO2 {}  Spr {}",
            self.zones.recovery.display(self.unit),
            self.zones.aerobic.display(self.unit),
            self.zones.steady.display(self.unit),
            self.zones.threshold.display(self.unit),
            self.zones.vo2.display(self.unit),
            self.zones.sprint.display(self.unit)
        )
    }
}

fn offset_pace(css_sec_per_100m: f64, offset: f64) -> Result<Pace, Error> {
    Pace::per_100m(css_sec_per_100m + offset)
}

fn range(css_sec_per_100m: f64, slow_offset: f64, fast_offset: f64) -> Result<PaceRange, Error> {
    Ok(PaceRange {
        easy_end: offset_pace(css_sec_per_100m, slow_offset)?,
        hard_end: offset_pace(css_sec_per_100m, fast_offset)?,
    })
}

/// Training zones from an already-known CSS.
///
/// ```
/// use sportanalytics::swimming::{css_from_trials, training_zones_from_css, Course, Event, Stroke, SwimTime};
///
/// let short = SwimTime::from_secs(Event::M200, Course::Scm, Stroke::Free, 150.0).unwrap();
/// let long = SwimTime::from_secs(Event::M400, Course::Scm, Stroke::Free, 368.0).unwrap();
/// let z = training_zones_from_css(css_from_trials(short, long).unwrap()).unwrap();
/// assert!(z.threshold.easy_end.sec_per_100m() > z.threshold.hard_end.sec_per_100m());
/// ```
pub fn training_zones_from_css(css: Css) -> Result<TrainingZones, Error> {
    let base = css.pace()?.sec_per_100m();
    Ok(TrainingZones {
        css,
        recovery: range(base, 25.0, 15.0)?,
        aerobic: range(base, 15.0, 8.0)?,
        steady: range(base, 8.0, 4.0)?,
        threshold: range(base, 2.0, -2.0)?,
        vo2: range(base, -3.0, -8.0)?,
        sprint: range(base, -8.0, -15.0)?,
    })
}

/// Training zones from two maximal CSS trials.
pub fn training_zones(short: SwimTime, long: SwimTime) -> Result<TrainingZones, Error> {
    training_zones_from_css(css_from_trials(short, long)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swimming::Pace;

    #[test]
    fn threshold_offsets_from_100_sec_css() {
        let css = Css {
            velocity_m_s: 1.0,
            adc_m: None,
        };
        // Construct via pace path: 100 s/100m ⇒ v = 1 m/s
        assert!((css.pace().unwrap().sec_per_100m() - 100.0).abs() < 1e-12);
        let z = training_zones_from_css(css).unwrap();
        assert!((z.threshold.easy_end.sec_per_100m() - 102.0).abs() < 1e-12);
        assert!((z.threshold.hard_end.sec_per_100m() - 98.0).abs() < 1e-12);
        assert!((z.recovery.easy_end.sec_per_100m() - 125.0).abs() < 1e-12);
        assert!((z.sprint.hard_end.sec_per_100m() - 85.0).abs() < 1e-12);
    }

    #[test]
    fn display_default_per_100m() {
        let css_pace = Pace::per_100m(100.0).unwrap();
        let css = Css {
            velocity_m_s: css_pace.velocity_m_per_s(),
            adc_m: None,
        };
        let z = training_zones_from_css(css).unwrap();
        let s = z.threshold.to_string();
        assert!(s.contains("/100m"), "got {s}");
    }
}
