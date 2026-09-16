//! USATF MLDR 2025 road age grading.
//!
//! This module looks up the official USATF Masters Long Distance Running (MLDR)
//! 2025 single-year road tables (Alan Jones / Tom Bernhard, approved
//! 2025-01-10, CC0). It is **not** an approximation and **not** championship
//! software of record, but it uses the same published road table as Howard
//! Grubb’s MLDR 2025 calculator.
//!
//! ```text
//! age_factor(age, sex, distance) ∈ (0, 1]   // 1.0 in open/prime years
//! age_standard  = open_standard / age_factor
//! age_grade %   = 100 × open_standard / (actual_time × age_factor)
//!               = 100 × age_standard / actual_time
//! open_equivalent = actual_time × age_factor
//! time_at_age_n   = actual_time × age_factor(current) / age_factor(n)
//! ```
//!
//! Ages must be in **5..=99**. Distances outside the official event span (about
//! 1 mile through 200 km), and road 3K (no 2025 file), return
//! [`Error::UnsupportedAgeGradeDistance`](crate::Error::UnsupportedAgeGradeDistance).
//! Off-grid distances interpolate **age standards** in log-distance between the
//! neighbouring official events (Jones 2025), not factors and not linear metres.

use std::fmt;

use super::time::format_hms;
use super::{Distance, RaceTime};
use crate::Error;

mod mldr_2025;
mod table;

/// Which published age-grade table to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AgeGradeTable {
    /// USATF MLDR 2025 road tables (Alan Jones / Tom Bernhard, approved 2025-01-10).
    UsatfMldr2025,
}

/// WMA/USATF male/female table standards, not a general gender model.
///
/// World Masters Athletics and USATF MLDR age-grade tables are published as two
/// sex categories. [`Gender::Male`] and [`Gender::Female`] select those open
/// standards and age factors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Gender {
    /// Male open standards and age factors.
    Male,
    /// Female open standards and age factors.
    Female,
}

/// Age-graded performance and optional equivalent time at another age.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AgeGradeResult {
    /// Age-graded percentage (`100 × age_standard / actual_time`).
    pub percent: f64,
    /// Informal community band for [`Self::percent`] (not an official WMA award).
    pub level: PerformanceLevel,
    /// Age factor for the athlete's current age and race distance.
    pub age_factor: f64,
    /// Open-class equivalent time (seconds): actual × factor.
    pub open_equivalent_secs: f64,
    /// Age standard time in seconds (`open_standard / age_factor`).
    pub age_standard_secs: f64,
    /// Table revision used for this result.
    pub table: AgeGradeTable,
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

/// Informal community bands for an age-graded percentage.
///
/// These thresholds are widely used in running communities. They are **not**
/// official World Masters Athletics or USATF award categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Map a percentage onto the usual informal age-grade bands.
    ///
    /// ```
    /// use sportanalytics::running::PerformanceLevel;
    ///
    /// assert_eq!(PerformanceLevel::from_percent(91.0), PerformanceLevel::WorldClass);
    /// ```
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

/// Open (prime-age) standard time in seconds from the given table revision.
///
/// Values come from the same USATF MLDR 2025 RunScore files as the age factors
/// (for example male 5K `769` s = 12:49, male marathon `7235` s = 2:00:35).
/// [`Distance::ThreeK`] is unsupported. Custom distances between official
/// events use Jones 2025 log-distance interpolation of the neighbouring opens.
///
/// ```
/// use sportanalytics::running::{open_standard_secs, Distance, Gender};
///
/// assert_eq!(
///     open_standard_secs(Distance::Marathon, Gender::Male).unwrap(),
///     7235.0
/// );
/// ```
pub fn open_standard_secs(distance: Distance, gender: Gender) -> Result<f64, Error> {
    open_standard_secs_with(distance, gender, AgeGradeTable::UsatfMldr2025)
}

/// Open standard for an explicit [`AgeGradeTable`].
pub fn open_standard_secs_with(
    distance: Distance,
    gender: Gender,
    table: AgeGradeTable,
) -> Result<f64, Error> {
    match table {
        AgeGradeTable::UsatfMldr2025 => table::open_standard_mldr2025(distance, gender),
    }
}

/// Age factor in `(0, 1]` for `age`, `gender`, and `distance` from the default
/// table ([`AgeGradeTable::UsatfMldr2025`]).
///
/// ```
/// use sportanalytics::running::{age_factor, Distance, Gender};
///
/// let f = age_factor(28, Gender::Male, Distance::FiveK).unwrap();
/// assert!((f - 1.0).abs() < 1e-9);
/// ```
pub fn age_factor(age: u8, gender: Gender, distance: Distance) -> Result<f64, Error> {
    age_factor_with(age, gender, distance, AgeGradeTable::UsatfMldr2025)
}

/// Age factor for an explicit [`AgeGradeTable`].
pub fn age_factor_with(
    age: u8,
    gender: Gender,
    distance: Distance,
    table: AgeGradeTable,
) -> Result<f64, Error> {
    Ok(table::lookup(table, distance, age, gender)?.factor)
}

/// Age-grade a performance with the default USATF MLDR 2025 table.
///
/// ```
/// use sportanalytics::running::{age_grade, Distance, Gender, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
/// let ag = age_grade(five, 42, Gender::Male, Some(25)).unwrap();
/// assert!((ag.percent - 68.6485).abs() < 0.1);
/// ```
pub fn age_grade(
    race: RaceTime,
    age: u8,
    gender: Gender,
    equivalent_age: Option<u8>,
) -> Result<AgeGradeResult, Error> {
    age_grade_with(
        race,
        age,
        gender,
        equivalent_age,
        AgeGradeTable::UsatfMldr2025,
    )
}

/// Age-grade a performance with an explicit [`AgeGradeTable`].
pub fn age_grade_with(
    race: RaceTime,
    age: u8,
    gender: Gender,
    equivalent_age: Option<u8>,
    table: AgeGradeTable,
) -> Result<AgeGradeResult, Error> {
    let looked = table::lookup(table, race.distance(), age, gender)?;
    let factor = looked.factor;
    let age_standard_secs = looked.age_standard_secs;
    let percent = 100.0 * age_standard_secs / race.seconds();
    let open_eq = race.seconds() * factor;
    let eq = match equivalent_age {
        Some(n) => {
            let f_n = table::lookup(table, race.distance(), n, gender)?.factor;
            Some(race.seconds() * factor / f_n)
        }
        None => None,
    };
    Ok(AgeGradeResult {
        percent,
        level: PerformanceLevel::from_percent(percent),
        age_factor: factor,
        open_equivalent_secs: open_eq,
        age_standard_secs,
        table,
        equivalent_at_age_secs: eq,
        equivalent_age,
    })
}

/// Same performance, expressed as a finish time at age `n`.
///
/// ```
/// use sportanalytics::running::{age_equivalent, Distance, Gender, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
/// let as_25 = age_equivalent(five, 42, Gender::Male, 25).unwrap();
/// assert!(as_25 < five.seconds());
/// ```
pub fn age_equivalent(
    race: RaceTime,
    age: u8,
    gender: Gender,
    target_age: u8,
) -> Result<f64, Error> {
    age_equivalent_with(race, age, gender, target_age, AgeGradeTable::UsatfMldr2025)
}

/// Age-equivalent time for an explicit [`AgeGradeTable`].
pub fn age_equivalent_with(
    race: RaceTime,
    age: u8,
    gender: Gender,
    target_age: u8,
    table: AgeGradeTable,
) -> Result<f64, Error> {
    let f = table::lookup(table, race.distance(), age, gender)?.factor;
    let f_n = table::lookup(table, race.distance(), target_age, gender)?.factor;
    Ok(race.seconds() * f / f_n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn open_headers_match_generated_table() {
        assert_eq!(
            open_standard_secs(Distance::FiveK, Gender::Male).unwrap(),
            769.0
        );
        assert_eq!(
            open_standard_secs(Distance::FiveK, Gender::Female).unwrap(),
            834.0
        );
        assert_eq!(
            open_standard_secs(Distance::Marathon, Gender::Male).unwrap(),
            7235.0
        );
    }

    #[test]
    fn per_event_factors_differ() {
        let five = age_factor(60, Gender::Male, Distance::FiveK).unwrap();
        let mara = age_factor(60, Gender::Male, Distance::Marathon).unwrap();
        assert!(
            (five - mara).abs() > 1e-6,
            "5K={five} marathon={mara} should differ"
        );
    }

    #[test]
    fn age_out_of_range() {
        assert_eq!(
            age_factor(4, Gender::Male, Distance::FiveK),
            Err(Error::AgeOutOfRange)
        );
        assert_eq!(
            age_factor(100, Gender::Male, Distance::FiveK),
            Err(Error::AgeOutOfRange)
        );
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        assert_eq!(
            age_grade(race, 4, Gender::Male, None),
            Err(Error::AgeOutOfRange)
        );
        assert_eq!(
            age_grade(race, 100, Gender::Male, None),
            Err(Error::AgeOutOfRange)
        );
    }

    #[test]
    fn three_k_unsupported() {
        assert_eq!(
            open_standard_secs(Distance::ThreeK, Gender::Male),
            Err(Error::UnsupportedAgeGradeDistance)
        );
        assert_eq!(
            age_factor(40, Gender::Male, Distance::ThreeK),
            Err(Error::UnsupportedAgeGradeDistance)
        );
    }

    #[test]
    fn ages_5_and_99_in_range() {
        assert!(age_factor(5, Gender::Male, Distance::FiveK).unwrap() > 0.0);
        assert!(age_factor(99, Gender::Male, Distance::FiveK).unwrap() > 0.0);
    }

    #[test]
    fn eight_k_standard_between_5k_and_10k() {
        // Official AgeGrade.8k row (exact lookup), sanity vs neighbours.
        let age = 40_u8;
        let s = |d: Distance| {
            let open = open_standard_secs(d, Gender::Male).unwrap();
            let f = age_factor(age, Gender::Male, d).unwrap();
            open / f
        };
        let s5 = s(Distance::FiveK);
        let s8 = s(Distance::custom(8_000.0, "8K").unwrap());
        let s10 = s(Distance::TenK);
        assert!(s8 > s5 && s8 < s10, "s5={s5} s8={s8} s10={s10}");
    }

    #[test]
    fn custom_log_interpolation_between_neighbours() {
        // 7K is not an official file; Jones log-distance interpolation of standards.
        let age = 40_u8;
        let seven = Distance::custom(7_000.0, "7K").unwrap();
        let lookup7 =
            table::lookup(AgeGradeTable::UsatfMldr2025, seven, age, Gender::Male).unwrap();
        let low = Distance::custom(4.0 * super::super::units::METERS_PER_MILE, "4mi").unwrap();
        let high = Distance::custom(8_000.0, "8K").unwrap();
        let s_at = |d: Distance| {
            let open = open_standard_secs(d, Gender::Male).unwrap();
            open / age_factor(age, Gender::Male, d).unwrap()
        };
        let s_low = s_at(low);
        let s_high = s_at(high);
        assert!(lookup7.age_standard_secs > s_low);
        assert!(lookup7.age_standard_secs < s_high);
        // Not the old linear-in-metres open with a shared (distance-free) factor.
        let u_lin = (7_000.0 - low.meters()) / (high.meters() - low.meters());
        let open_low = open_standard_secs(low, Gender::Male).unwrap();
        let open_high = open_standard_secs(high, Gender::Male).unwrap();
        let fake_open = open_low + u_lin * (open_high - open_low);
        let shared_f = age_factor(age, Gender::Male, Distance::FiveK).unwrap();
        let fake_std = fake_open / shared_f;
        assert!(
            (lookup7.age_standard_secs - fake_std).abs() > 0.05,
            "expected Jones log-std interpolation to differ from linear-open/shared-factor"
        );
    }

    #[test]
    fn outside_official_span_errors() {
        let short = Distance::custom(1_000.0, "1K").unwrap();
        let long = Distance::custom(250_000.0, "250K").unwrap();
        assert_eq!(
            age_factor(40, Gender::Male, short),
            Err(Error::UnsupportedAgeGradeDistance)
        );
        assert_eq!(
            age_factor(40, Gender::Male, long),
            Err(Error::UnsupportedAgeGradeDistance)
        );
    }

    /// Golden cases vs Howard Grubb MLDR 2025 road calculator
    /// (<https://howardgrubb.co.uk/athletics/mldrroad25.html>), recorded 2026-09-16.
    #[test]
    fn grubb_oracle_cases() {
        let cases = [
            // (gender, age, distance, h, m, s, factor, percent)
            (Gender::Male, 42, Distance::FiveK, 0, 20, 0, 0.9335, 68.6485),
            (
                Gender::Female,
                42,
                Distance::FiveK,
                0,
                22,
                0,
                0.9346,
                67.6031,
            ),
            (Gender::Male, 28, Distance::FiveK, 0, 20, 0, 1.0000, 64.0833),
            (
                Gender::Male,
                60,
                Distance::Marathon,
                3,
                30,
                0,
                0.8143,
                70.5153,
            ),
            (
                Gender::Female,
                55,
                Distance::HalfMarathon,
                1,
                45,
                0,
                0.8235,
                72.7055,
            ),
        ];
        for (gender, age, distance, h, m, s, want_f, want_pct) in cases {
            let race = RaceTime::from_hms(distance, h, m, s).unwrap();
            let ag = age_grade(race, age, gender, None).unwrap();
            assert!(
                (ag.age_factor - want_f).abs() < 5e-4,
                "{gender:?} {age} {distance}: factor {} vs {want_f}",
                ag.age_factor
            );
            assert!(
                (ag.percent - want_pct).abs() < 0.1,
                "{gender:?} {age} {distance}: percent {} vs {want_pct}",
                ag.percent
            );
            assert_eq!(ag.table, AgeGradeTable::UsatfMldr2025);
            assert!(ag.age_standard_secs > 0.0);
        }
    }

    #[test]
    fn equivalent_age_round_trip() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let ag = age_grade(race, 42, Gender::Male, Some(25)).unwrap();
        let as_25 = ag.equivalent_at_age_secs.unwrap();
        let back = age_equivalent(
            RaceTime::from_secs(Distance::FiveK, as_25).unwrap(),
            25,
            Gender::Male,
            42,
        )
        .unwrap();
        assert!((back - race.seconds()).abs() < 1e-6);
        let direct = age_equivalent(race, 42, Gender::Male, 25).unwrap();
        assert!((direct - as_25).abs() < 1e-9);
    }

    #[test]
    fn prime_age_open_equivalent_is_the_race_time() {
        let race = RaceTime::from_hms(Distance::TenK, 0, 40, 0).unwrap();
        let ag = age_grade(race, 25, Gender::Female, None).unwrap();
        assert!((ag.age_factor - 1.0).abs() < 1e-9);
        assert!((ag.open_equivalent_secs - race.seconds()).abs() < 1e-9);
        assert_eq!(ag.open_equivalent_hms(), "40:00");
    }

    #[test]
    fn age_grade_display_includes_percent_and_open_eq() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let s = age_grade(race, 42, Gender::Male, Some(25))
            .unwrap()
            .to_string();
        assert!(s.contains('%'));
        assert!(s.contains("open eq"));
        assert!(s.contains("25yo"));
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
    }

    #[test]
    fn open_standards_ordered_for_supported_named() {
        for gender in [Gender::Male, Gender::Female] {
            let mut prev = 0.0;
            for d in [
                Distance::FiveK,
                Distance::TenK,
                Distance::HalfMarathon,
                Distance::Marathon,
            ] {
                let s = open_standard_secs(d, gender).unwrap();
                assert!(s > prev);
                prev = s;
            }
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_age_grade_roundtrip() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let ag = age_grade(race, 42, Gender::Male, Some(25)).unwrap();
        let back: AgeGradeResult =
            serde_json::from_str(&serde_json::to_string(&ag).unwrap()).unwrap();
        assert!((back.percent - ag.percent).abs() < 1e-6);
        assert_eq!(back.level, ag.level);
        assert_eq!(back.table, AgeGradeTable::UsatfMldr2025);
    }
}
