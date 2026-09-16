//! Daniels training zones from VDOT.
//!
//! Cameron and Riegel times do not define zones. Use the VDOT from a race
//! (or the mean/best of several races via [`super::vo2max_from_races`]).
//!
//! Typical Daniels bands:
//!
//! | Zone | % of VDOT | Use |
//! |------|-----------|-----|
//! | Easy (E) | 59–74% | easy / long run |
//! | Marathon (M) | 75–84% | marathon pace |
//! | Threshold (T) | 83–88% | tempo / cruise intervals |
//! | Interval (I) | 95–100% | 3–5 min VO2 reps |
//! | Repetition (R) | ~105–110% | short fast reps |
//!
//! Target VO2 = `vdot * pct`, then [`super::velocity_from_vo2`].
//! Paces are computed from those equations, not copied from Daniels’ published
//! (copyrighted) pace tables.

use super::vo2::{vdot, velocity_from_vo2};
use super::{RaceTime, Vdot};

/// A pace band in seconds per kilometre.
///
/// `easy_end` is the slower edge; `hard_end` is the faster edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaceRange {
    /// Slower edge (sec / km).
    pub easy_end: f64,
    /// Faster edge (sec / km).
    pub hard_end: f64,
}

/// Daniels training zones derived from a VDOT.
#[derive(Debug, Clone, PartialEq)]
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

fn pace_sec_per_km(vdot: f64, pct: f64) -> f64 {
    let v_m_per_min = velocity_from_vo2(vdot * pct);
    1000.0 / v_m_per_min * 60.0
}

fn range(vdot: f64, lo: f64, hi: f64) -> PaceRange {
    PaceRange {
        easy_end: pace_sec_per_km(vdot, lo),
        hard_end: pace_sec_per_km(vdot, hi),
    }
}

/// Training zones from an already-known VDOT.
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
pub fn training_zones(race: RaceTime) -> TrainingZones {
    training_zones_from_vdot(vdot(race))
}

/// Format a pace as `m:ss /km`.
pub fn format_pace(sec_per_km: f64) -> String {
    let total = sec_per_km.round() as u64;
    format!("{}:{:02} /km", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::running::Distance;

    fn parse_pace(label: &str) -> f64 {
        let (m, s) = label.split_once(':').unwrap();
        m.parse::<f64>().unwrap() * 60.0 + s.parse::<f64>().unwrap()
    }

    #[test]
    fn twenty_min_5k_matches_published_bands() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let z = training_zones(race);
        assert!((z.vdot.value() - 49.8).abs() < 0.4);

        // Invert at %VDOT (not a copied pace chart). Easy is a wide 59–74%
        // band; T/I sit near the usual published ballpark for VDOT ~50.
        assert_eq!(format_pace(z.easy.hard_end), "4:54 /km");
        assert_eq!(format_pace(z.easy.easy_end), "5:53 /km");
        assert!(
            (z.threshold.hard_end - parse_pace("4:15")).abs() < 5.0,
            "T hard {}",
            z.threshold.hard_end
        );
        assert!(
            (z.threshold.easy_end - parse_pace("4:26")).abs() < 5.0,
            "T easy {}",
            z.threshold.easy_end
        );
        assert!(
            (z.interval.hard_end - parse_pace("3:56")).abs() < 8.0,
            "I hard {}",
            z.interval.hard_end
        );
        assert!(
            (z.interval.easy_end - parse_pace("4:05")).abs() < 8.0,
            "I easy {}",
            z.interval.easy_end
        );
    }

    #[test]
    fn slower_edge_is_slower_than_faster_edge() {
        let race = RaceTime::from_hms(Distance::TenK, 0, 42, 0).unwrap();
        let z = training_zones(race);
        for band in [z.easy, z.marathon, z.threshold, z.interval, z.repetition] {
            assert!(
                band.easy_end > band.hard_end,
                "easy_end {} should be slower (larger) than hard_end {}",
                band.easy_end,
                band.hard_end
            );
        }
        assert!(z.easy.easy_end > z.marathon.easy_end);
        assert!(z.threshold.hard_end > z.interval.hard_end);
        assert!(z.interval.hard_end > z.repetition.hard_end);
    }

    #[test]
    fn from_vdot_matches_from_race() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let from_race = training_zones(race);
        let from_vdot = training_zones_from_vdot(from_race.vdot);
        assert_eq!(from_race, from_vdot);
    }

    #[test]
    fn format_pace_rounds_to_mm_ss() {
        assert_eq!(format_pace(307.4), "5:07 /km");
        assert_eq!(format_pace(307.6), "5:08 /km");
        assert_eq!(format_pace(60.0), "1:00 /km");
    }
}
