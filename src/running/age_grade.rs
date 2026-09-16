//! WMA-style age grading.
//!
//! Official World Masters Athletics / USATF MLDR tables are large lookup grids.
//! This module ships a compact interpolated approximation so the crate stays
//! dependency-free. Treat percentages as estimates, not championship scoring.
//!
//! ```text
//! age_factor(age, sex) ∈ (0, 1]          // 1.0 in open/prime years
//! age_standard         = open_standard / age_factor
//! age_grade %          = 100 × age_standard / actual_time
//! open_equivalent      = actual_time × age_factor
//! time_at_age_n        = actual_time × age_factor(current) / age_factor(n)
//! ```

use std::fmt;

use super::time::format_hms;
use super::{Distance, RaceTime};

/// Sex used by the age-grading tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    /// Male / men open standards and age factors.
    Male,
    /// Female / women open standards and age factors.
    Female,
}

/// Age-graded performance and optional equivalent time at another age.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgeGradeResult {
    /// Age-graded percentage (`100 × age_standard / actual_time`).
    pub percent: f64,
    /// Banded interpretation of [`Self::percent`].
    pub level: PerformanceLevel,
    /// Interpolated age factor for the athlete's current age.
    pub age_factor: f64,
    /// Open-class equivalent time (seconds): actual × factor.
    pub open_equivalent_secs: f64,
    /// Time that preserves the same % at `equivalent_age` (seconds).
    pub equivalent_at_age_secs: Option<f64>,
    /// Age requested for [`Self::equivalent_at_age_secs`].
    pub equivalent_age: Option<u8>,
}

impl AgeGradeResult {
    /// Open-class equivalent time formatted as `h:mm:ss` or `m:ss`.
    pub fn open_equivalent_hms(&self) -> String {
        format_hms(self.open_equivalent_secs)
    }

    /// Equivalent time at [`Self::equivalent_age`], if requested.
    pub fn equivalent_at_age_hms(&self) -> Option<String> {
        self.equivalent_at_age_secs.map(format_hms)
    }
}

impl fmt::Display for AgeGradeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.1}% {} | open eq {}",
            self.percent,
            self.level.label(),
            self.open_equivalent_hms()
        )?;
        if let (Some(age), Some(hms)) = (self.equivalent_age, self.equivalent_at_age_hms()) {
            write!(f, " | as {age}yo {hms}")?;
        }
        Ok(())
    }
}

/// Qualitative band for an age-graded percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceLevel {
    /// ≥ 100%.
    WorldRecord,
    /// ≥ 90%.
    WorldClass,
    /// ≥ 80%.
    NationalClass,
    /// ≥ 70%.
    RegionalClass,
    /// ≥ 60%.
    LocalClass,
    /// ≥ 50%.
    Recreational,
    /// < 50%.
    Developing,
}

impl PerformanceLevel {
    /// Map a percentage onto the usual age-grade bands.
    pub fn from_percent(p: f64) -> Self {
        if p >= 100.0 {
            Self::WorldRecord
        } else if p >= 90.0 {
            Self::WorldClass
        } else if p >= 80.0 {
            Self::NationalClass
        } else if p >= 70.0 {
            Self::RegionalClass
        } else if p >= 60.0 {
            Self::LocalClass
        } else if p >= 50.0 {
            Self::Recreational
        } else {
            Self::Developing
        }
    }

    /// Human-readable label for this band.
    pub fn label(self) -> &'static str {
        match self {
            Self::WorldRecord => "world-record level",
            Self::WorldClass => "world class",
            Self::NationalClass => "national class",
            Self::RegionalClass => "regional class",
            Self::LocalClass => "local class",
            Self::Recreational => "recreational",
            Self::Developing => "developing",
        }
    }
}

/// Approximate 2025-era open (prime-age) road standards, seconds.
///
/// 5K–marathon aligned with commonly cited WMA/USATF open standards.
/// 3K uses a track-adjacent open standard (no official road 3K table).
/// [`Distance::Custom`] is linearly interpolated (or extrapolated) in metres
/// between those named standards.
pub fn open_standard_secs(distance: Distance, gender: Gender) -> f64 {
    match distance {
        Distance::Custom { meters, .. } => interpolate_open_standard(meters, gender),
        named => named_open_standard_secs(named, gender),
    }
}

fn named_open_standard_secs(distance: Distance, gender: Gender) -> f64 {
    match (gender, distance) {
        (Gender::Male, Distance::ThreeK) => 440.0,         // 7:20
        (Gender::Male, Distance::FiveK) => 769.0,          // 12:49
        (Gender::Male, Distance::TenK) => 1_584.0,         // 26:24
        (Gender::Male, Distance::HalfMarathon) => 3_451.0, // 57:31
        (Gender::Male, Distance::Marathon) => 7_235.0,     // 2:00:35
        (Gender::Female, Distance::ThreeK) => 500.0,       // 8:20
        (Gender::Female, Distance::FiveK) => 834.0,        // 13:54
        (Gender::Female, Distance::TenK) => 1_726.0,       // 28:46
        (Gender::Female, Distance::HalfMarathon) => 3_772.0, // 1:02:52
        (Gender::Female, Distance::Marathon) => 7_796.0,   // 2:09:56
        (_, Distance::Custom { .. }) => unreachable!("named distances only"),
    }
}

fn interpolate_open_standard(meters: f64, gender: Gender) -> f64 {
    let named = Distance::all();
    let std = |d: Distance| named_open_standard_secs(d, gender);
    if meters <= named[0].meters() {
        return lerp(
            named[0].meters(),
            std(named[0]),
            named[1].meters(),
            std(named[1]),
            meters,
        );
    }
    for pair in named.windows(2) {
        if meters <= pair[1].meters() {
            return lerp(
                pair[0].meters(),
                std(pair[0]),
                pair[1].meters(),
                std(pair[1]),
                meters,
            );
        }
    }
    let last = named.len() - 1;
    lerp(
        named[last - 1].meters(),
        std(named[last - 1]),
        named[last].meters(),
        std(named[last]),
        meters,
    )
}

fn lerp(x0: f64, y0: f64, x1: f64, y1: f64, x: f64) -> f64 {
    let t = (x - x0) / (x1 - x0);
    y0 + t * (y1 - y0)
}

/// Compact age-factor knots (age, factor). Linearly interpolated.
/// Shape follows typical WMA road decline; not the official table.
fn factor_knots(gender: Gender) -> &'static [(u8, f64)] {
    match gender {
        Gender::Male => &[
            (8, 0.82),
            (15, 0.94),
            (20, 1.00),
            (34, 1.00),
            (40, 0.950),
            (45, 0.918),
            (50, 0.882),
            (55, 0.842),
            (60, 0.798),
            (65, 0.745),
            (70, 0.682),
            (75, 0.605),
            (80, 0.515),
            (85, 0.415),
            (90, 0.320),
            (100, 0.210),
        ],
        Gender::Female => &[
            (8, 0.80),
            (15, 0.93),
            (20, 1.00),
            (34, 1.00),
            (40, 0.958),
            (45, 0.920),
            (50, 0.872),
            (55, 0.818),
            (60, 0.755),
            (65, 0.685),
            (70, 0.605),
            (75, 0.515),
            (80, 0.420),
            (85, 0.330),
            (90, 0.250),
            (100, 0.160),
        ],
    }
}

/// Age factor in `(0, 1]`. Prime years (20–34) are `1.0`. Ages are clamped to 8–100.
pub fn age_factor(age: u8, gender: Gender) -> f64 {
    let knots = factor_knots(gender);
    let age = age.clamp(8, 100);
    if age <= knots[0].0 {
        return knots[0].1;
    }
    for w in knots.windows(2) {
        let (a0, f0) = w[0];
        let (a1, f1) = w[1];
        if age <= a1 {
            let t = (age - a0) as f64 / (a1 - a0) as f64;
            return f0 + t * (f1 - f0);
        }
    }
    knots.last().unwrap().1
}

/// Age-grade a performance and optionally convert it to an equivalent time at age `n`.
pub fn age_grade(
    race: RaceTime,
    age: u8,
    gender: Gender,
    equivalent_age: Option<u8>,
) -> AgeGradeResult {
    let f = age_factor(age, gender);
    let open = open_standard_secs(race.distance(), gender);
    let age_std = open / f;
    let percent = 100.0 * age_std / race.seconds();
    let open_eq = race.seconds() * f;
    let eq = equivalent_age.map(|n| race.seconds() * f / age_factor(n, gender));
    AgeGradeResult {
        percent,
        level: PerformanceLevel::from_percent(percent),
        age_factor: f,
        open_equivalent_secs: open_eq,
        equivalent_at_age_secs: eq,
        equivalent_age,
    }
}

/// Same performance, expressed as a finish time at age `n`.
pub fn age_equivalent(race: RaceTime, age: u8, gender: Gender, target_age: u8) -> f64 {
    race.seconds() * age_factor(age, gender) / age_factor(target_age, gender)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime_age_factor_is_one() {
        assert!((age_factor(28, Gender::Male) - 1.0).abs() < 1e-9);
        assert!((age_factor(28, Gender::Female) - 1.0).abs() < 1e-9);
        assert!((age_factor(20, Gender::Male) - 1.0).abs() < 1e-9);
        assert!((age_factor(34, Gender::Female) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn older_runner_has_higher_percent_than_raw_wr_share() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 22, 0).unwrap();
        let ag = age_grade(race, 55, Gender::Male, Some(30));
        assert!(ag.percent > 55.0);
        assert!(ag.equivalent_at_age_secs.unwrap() < race.seconds());
        assert_eq!(ag.equivalent_age, Some(30));
        assert_eq!(ag.level, PerformanceLevel::from_percent(ag.percent));
    }

    #[test]
    fn prime_age_open_equivalent_is_the_race_time() {
        let race = RaceTime::from_hms(Distance::TenK, 0, 40, 0).unwrap();
        let ag = age_grade(race, 25, Gender::Female, None);
        assert!((ag.age_factor - 1.0).abs() < 1e-9);
        assert!((ag.open_equivalent_secs - race.seconds()).abs() < 1e-9);
        assert!(ag.equivalent_at_age_secs.is_none());
        assert_eq!(ag.open_equivalent_hms(), "40:00");
    }

    #[test]
    fn age_grade_display_includes_percent_and_open_eq() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let s = age_grade(race, 42, Gender::Male, Some(25)).to_string();
        assert!(s.contains('%'));
        assert!(s.contains("open eq"));
        assert!(s.contains("25yo"));
    }

    #[test]
    fn age_equivalent_matches_age_grade_option() {
        let race = RaceTime::from_hms(Distance::Marathon, 3, 10, 0).unwrap();
        let secs = age_equivalent(race, 60, Gender::Male, 30);
        let ag = age_grade(race, 60, Gender::Male, Some(30));
        assert!((secs - ag.equivalent_at_age_secs.unwrap()).abs() < 1e-9);
        assert!(ag.equivalent_at_age_hms().is_some());
    }

    #[test]
    fn factor_declines_after_prime() {
        let f40 = age_factor(40, Gender::Male);
        let f60 = age_factor(60, Gender::Male);
        let f80 = age_factor(80, Gender::Male);
        assert!(f40 < 1.0);
        assert!(f60 < f40);
        assert!(f80 < f60);
    }

    #[test]
    fn ages_are_clamped_to_knots() {
        assert_eq!(age_factor(0, Gender::Female), age_factor(8, Gender::Female));
        assert_eq!(age_factor(120, Gender::Male), age_factor(100, Gender::Male));
    }

    #[test]
    fn performance_level_thresholds() {
        assert_eq!(
            PerformanceLevel::from_percent(100.0),
            PerformanceLevel::WorldRecord
        );
        assert_eq!(
            PerformanceLevel::from_percent(90.0),
            PerformanceLevel::WorldClass
        );
        assert_eq!(
            PerformanceLevel::from_percent(80.0),
            PerformanceLevel::NationalClass
        );
        assert_eq!(
            PerformanceLevel::from_percent(70.0),
            PerformanceLevel::RegionalClass
        );
        assert_eq!(
            PerformanceLevel::from_percent(60.0),
            PerformanceLevel::LocalClass
        );
        assert_eq!(
            PerformanceLevel::from_percent(50.0),
            PerformanceLevel::Recreational
        );
        assert_eq!(
            PerformanceLevel::from_percent(49.9),
            PerformanceLevel::Developing
        );
        assert_eq!(PerformanceLevel::WorldRecord.label(), "world-record level");
        assert_eq!(PerformanceLevel::WorldClass.label(), "world class");
        assert_eq!(PerformanceLevel::NationalClass.label(), "national class");
        assert_eq!(PerformanceLevel::RegionalClass.label(), "regional class");
        assert_eq!(PerformanceLevel::LocalClass.label(), "local class");
        assert_eq!(PerformanceLevel::Recreational.label(), "recreational");
        assert_eq!(PerformanceLevel::Developing.label(), "developing");
    }

    #[test]
    fn open_standards_are_positive_and_ordered() {
        for gender in [Gender::Male, Gender::Female] {
            let mut prev = 0.0;
            for d in Distance::all() {
                let s = open_standard_secs(d, gender);
                assert!(s > prev);
                prev = s;
            }
        }
        assert!(
            open_standard_secs(Distance::FiveK, Gender::Female)
                > open_standard_secs(Distance::FiveK, Gender::Male)
        );
    }

    #[test]
    fn custom_open_standard_sits_between_named_neighbours() {
        let eight = Distance::custom(8_000.0, "8K").unwrap();
        let s = open_standard_secs(eight, Gender::Male);
        let five = open_standard_secs(Distance::FiveK, Gender::Male);
        let ten = open_standard_secs(Distance::TenK, Gender::Male);
        assert!(s > five);
        assert!(s < ten);
    }
}
