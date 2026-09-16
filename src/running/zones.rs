//! Daniels training zones from VDOT.
//!
//! Cameron and Riegel times do not define zones. Use the VDOT from a race
//! (or the mean/best of several races via [`super::vo2max_from_races`]).
//!
//! Fixed % of VDOT used by this crate (equation inversion, not a pace chart):
//!
//! | Zone | % of VDOT | Use |
//! |------|-----------|-----|
//! | Easy (E) | **0.59–0.74** | easy / long run |
//! | Marathon (M) | **0.75–0.84** | marathon pace |
//! | Threshold (T) | **0.83–0.88** | tempo / cruise intervals |
//! | Interval (I) | **0.95–1.00** | 3–5 min VO2 reps |
//! | Repetition (R) | **1.05–1.10** | short fast reps |
//!
//! Target VO2 = `vdot * pct`, then [`super::velocity_from_vo2`].
//! Paces are computed from those equations, not copied from Daniels’ published
//! (copyrighted) pace tables. Published Daniels *Running Formula* charts will
//! differ by a few seconds/km.

use std::fmt;

use super::units::{LengthUnit, Pace};
use super::vo2::{vdot, velocity_from_vo2};
use super::{RaceTime, Vdot};

/// A pace band with a slower and a faster edge.
///
/// `easy_end` is the slower edge; `hard_end` is the faster edge.
/// Default [`std::fmt::Display`] uses `/km`; use [`Self::display`] for `/mi`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PaceRange {
    /// Slower edge.
    pub easy_end: Pace,
    /// Faster edge.
    pub hard_end: Pace,
}

impl PaceRange {
    /// Format both edges in `unit` (`/km` or `/mi`).
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

/// Daniels training zones derived from a VDOT.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrainingZones {
    /// VDOT used to compute the zones.
    pub vdot: Vdot,
    /// Easy / long-run pace, 59–74% of VDOT.
    pub easy: PaceRange,
    /// Marathon-pace, 75–84% of VDOT.
    pub marathon: PaceRange,
    /// Threshold / tempo, 83–88% of VDOT.
    pub threshold: PaceRange,
    /// Interval (VO2) pace, 95–100% of VDOT.
    pub interval: PaceRange,
    /// Repetition pace, 105–110% of VDOT (supra-max approximation).
    pub repetition: PaceRange,
}

impl TrainingZones {
    /// Format the full E/M/T/I/R line in `unit` (`/km` or `/mi`).
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

fn pace_sec_per_km(vdot: f64, pct: f64) -> Pace {
    let v_m_per_min = velocity_from_vo2(vdot * pct);
    let sec_per_km = 1000.0 / v_m_per_min * 60.0;
    Pace::per_km(sec_per_km).expect("positive velocity implies positive pace")
}

fn range(vdot: f64, lo: f64, hi: f64) -> PaceRange {
    PaceRange {
        easy_end: pace_sec_per_km(vdot, lo),
        hard_end: pace_sec_per_km(vdot, hi),
    }
}

/// Training zones from an already-known VDOT.
///
/// ```
/// use sportanalytics::running::{training_zones_from_vdot, vdot, Distance, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
/// let z = training_zones_from_vdot(vdot(five));
/// assert!(z.easy.easy_end.sec_per_km() > z.easy.hard_end.sec_per_km());
/// ```
pub fn training_zones_from_vdot(vdot: Vdot) -> TrainingZones {
    let v = vdot.value();
    TrainingZones {
        vdot,
        easy: range(v, 0.59, 0.74),
        marathon: range(v, 0.75, 0.84),
        threshold: range(v, 0.83, 0.88),
        interval: range(v, 0.95, 1.00),
        repetition: range(v, 1.05, 1.10),
    }
}

/// Training zones from a single race result.
///
/// Edges are equation paces at the fixed %VDOT bands (E 0.59–0.74, …).
/// A 20:00 5K yields about VDOT 49.8; Easy is `5:53–4:54 /km`.
///
/// ```
/// use sportanalytics::running::{training_zones, Distance, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
/// let z = training_zones(five);
/// assert_eq!(z.easy.hard_end.to_string(), "4:54 /km");
/// assert_eq!(z.threshold.hard_end.to_string(), "4:16 /km");
/// assert_eq!(z.interval.hard_end.to_string(), "3:51 /km");
/// ```
pub fn training_zones(race: RaceTime) -> TrainingZones {
    training_zones_from_vdot(vdot(race))
}

/// Format a pace as `m:ss /km`.
///
/// Prefer [`Pace::per_km`] and [`Display`][`Pace`] (or [`Pace::display`]) instead.
///
/// ```
/// # #[allow(deprecated)]
/// use sportanalytics::running::format_pace;
///
/// # #[allow(deprecated)]
/// assert_eq!(format_pace(294.0), "4:54 /km");
/// ```
#[deprecated(note = "use Pace::per_km(sec)?.to_string() or Display")]
pub fn format_pace(sec_per_km: f64) -> String {
    let total = sec_per_km.round() as u64;
    format!("{}:{:02} /km", total / 60, total % 60)
}

impl fmt::Display for TrainingZones {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.display(LengthUnit::Kilometer), f)
    }
}

impl fmt::Display for TrainingZonesDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VDOT {}  E {}  M {}  T {}  I {}  R {}",
            self.zones.vdot,
            self.zones.easy.display(self.unit),
            self.zones.marathon.display(self.unit),
            self.zones.threshold.display(self.unit),
            self.zones.interval.display(self.unit),
            self.zones.repetition.display(self.unit)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::running::{velocity_from_vo2, Distance};

    fn pace_from_pct(vdot: f64, pct: f64) -> Pace {
        let v_m_per_min = velocity_from_vo2(vdot * pct);
        let sec_per_km = 1000.0 / v_m_per_min * 60.0;
        Pace::per_km(sec_per_km).unwrap()
    }

    #[test]
    fn twenty_min_5k_matches_equation_bands() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let z = training_zones(race);
        assert!((z.vdot.value() - 49.8).abs() < 0.05);

        // Equation paces at the frozen %VDOT edges (not a Running Formula cell).
        assert_eq!(z.easy.easy_end.to_string(), "5:53 /km");
        assert_eq!(z.easy.hard_end.to_string(), "4:54 /km");
        assert_eq!(z.threshold.easy_end.to_string(), "4:28 /km");
        assert_eq!(z.threshold.hard_end.to_string(), "4:16 /km");
        assert_eq!(z.interval.easy_end.to_string(), "4:01 /km");
        assert_eq!(z.interval.hard_end.to_string(), "3:51 /km");
    }

    #[test]
    fn vdot_50_zone_edges_strictly_faster_across_bands() {
        let z = training_zones_from_vdot(Vdot::new(50.0).unwrap());
        // Ordered by %VDOT so pace (sec/km) decreases; M/T overlap is intentional.
        let edges = [
            z.easy.easy_end.sec_per_km(),       // 0.59
            z.easy.hard_end.sec_per_km(),       // 0.74
            z.marathon.easy_end.sec_per_km(),   // 0.75
            z.threshold.easy_end.sec_per_km(),  // 0.83
            z.marathon.hard_end.sec_per_km(),   // 0.84
            z.threshold.hard_end.sec_per_km(),  // 0.88
            z.interval.easy_end.sec_per_km(),   // 0.95
            z.interval.hard_end.sec_per_km(),   // 1.00
            z.repetition.easy_end.sec_per_km(), // 1.05
            z.repetition.hard_end.sec_per_km(), // 1.10
        ];
        for w in edges.windows(2) {
            assert!(
                w[0] > w[1],
                "expected strictly slower→faster: {} then {}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn zone_edges_match_pace_from_pct() {
        let vd = Vdot::new(50.0).unwrap();
        let z = training_zones_from_vdot(vd);
        let v = vd.value();
        let pairs = [
            (z.easy.easy_end, 0.59),
            (z.easy.hard_end, 0.74),
            (z.marathon.easy_end, 0.75),
            (z.marathon.hard_end, 0.84),
            (z.threshold.easy_end, 0.83),
            (z.threshold.hard_end, 0.88),
            (z.interval.easy_end, 0.95),
            (z.interval.hard_end, 1.00),
            (z.repetition.easy_end, 1.05),
            (z.repetition.hard_end, 1.10),
        ];
        for (edge, pct) in pairs {
            let expected = pace_from_pct(v, pct);
            assert!(
                (edge.sec_per_km() - expected.sec_per_km()).abs() < 1e-9,
                "pct {pct}: got {} want {}",
                edge.sec_per_km(),
                expected.sec_per_km()
            );
        }
    }

    #[test]
    fn slower_edge_is_slower_than_faster_edge() {
        let race = RaceTime::from_hms(Distance::TenK, 0, 42, 0).unwrap();
        let z = training_zones(race);
        for band in [z.easy, z.marathon, z.threshold, z.interval, z.repetition] {
            assert!(
                band.easy_end.sec_per_km() > band.hard_end.sec_per_km(),
                "easy_end {} should be slower (larger) than hard_end {}",
                band.easy_end.sec_per_km(),
                band.hard_end.sec_per_km()
            );
        }
        assert!(z.easy.easy_end.sec_per_km() > z.marathon.easy_end.sec_per_km());
        assert!(z.threshold.hard_end.sec_per_km() > z.interval.hard_end.sec_per_km());
        assert!(z.interval.hard_end.sec_per_km() > z.repetition.hard_end.sec_per_km());
    }

    #[test]
    fn from_vdot_matches_from_race() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let from_race = training_zones(race);
        let from_vdot = training_zones_from_vdot(from_race.vdot);
        assert_eq!(from_race, from_vdot);
    }

    #[test]
    #[allow(deprecated)]
    fn format_pace_rounds_to_mm_ss() {
        assert_eq!(format_pace(307.4), "5:07 /km");
        assert_eq!(format_pace(307.6), "5:08 /km");
        assert_eq!(format_pace(60.0), "1:00 /km");
    }

    #[test]
    fn training_zones_display_lists_bands() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let z = training_zones(race);
        let s = z.to_string();
        assert!(s.contains("E "));
        assert!(s.contains("T "));
        assert!(s.contains("/km"));
        let mi = z.display(LengthUnit::Mile).to_string();
        assert!(mi.contains("/mi"));
        assert_eq!(
            z.easy
                .display(LengthUnit::Mile)
                .to_string()
                .matches("/mi")
                .count(),
            2
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_training_zones_roundtrip() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let zones = training_zones(race);
        let back: TrainingZones =
            serde_json::from_str(&serde_json::to_string(&zones).unwrap()).unwrap();
        assert!((back.vdot.value() - zones.vdot.value()).abs() < 1e-12);
        assert!((back.easy.hard_end.sec_per_km() - zones.easy.hard_end.sec_per_km()).abs() < 1e-6);
    }
}
