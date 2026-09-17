//! Age-factor helper for FTP (trained-endurance decline curve).
//!
//! There is no USATF-MLDR-quality open cycling age-grade table in this crate.
//! This module applies a cited **decline approximation** for trained endurance
//! athletes (plateau into the mid-30s, then about **0.5% per year** linear /
//! ~5% per decade for aerobic power). See Tanaka & Seals and related
//! endurance-aging literature. **Not** official age grading; sex is unused.
//!
//! Supported ages: **15..=90** ([`Error::AgeOutOfRange`] `{ min: 15, max: 90 }`
//! otherwise).

use super::ftp::Ftp;
use super::Error;

/// Age (years) at which the decline curve plateaus.
pub const AGE_PLATEAU: u16 = 35;
/// Fractional aerobic-power decline per year after [`AGE_PLATEAU`] (`0.005`).
pub const AEROBIC_DECLINE_PER_YEAR: f64 = 0.005;

const AGE_LO: u16 = 15;
const AGE_HI: u16 = 90;

fn check_age(age: u16) -> Result<(), Error> {
    if (AGE_LO..=AGE_HI).contains(&age) {
        Ok(())
    } else {
        Err(Error::AgeOutOfRange {
            min: AGE_LO,
            max: AGE_HI,
        })
    }
}

/// Linear decline: `1 − 0.005 × years_after_plateau` (matches “~0.5%/year”).
fn decline(age: u16) -> f64 {
    let years_after = age.max(AGE_PLATEAU).saturating_sub(AGE_PLATEAU) as f64;
    1.0 - AEROBIC_DECLINE_PER_YEAR * years_after
}

/// Multiplicative factor mapping performance at `age` to `reference_age`.
///
/// `factor = decline(age) / decline(reference_age)`. Ages outside 15..=90
/// return [`Error::AgeOutOfRange`] `{ min: 15, max: 90 }`.
pub fn age_factor(age: u16, reference_age: u16) -> Result<f64, Error> {
    check_age(age)?;
    check_age(reference_age)?;
    Ok(decline(age) / decline(reference_age))
}

/// Age-equivalent FTP at `reference_age` from current FTP at `age`.
///
/// `ftp_ref = ftp_now / factor(age → reference_age)`.
///
/// ```
/// use sportanalytics::cycling::{age_equivalent_ftp, Ftp};
///
/// let eq = age_equivalent_ftp(Ftp::new(250.0).unwrap(), 55, 35).unwrap();
/// assert!((eq.watts() - 277.8).abs() < 0.5);
/// ```
pub fn age_equivalent_ftp(ftp: Ftp, age: u16, reference_age: u16) -> Result<Ftp, Error> {
    let factor = age_factor(age, reference_age)?;
    Ftp::new(ftp.watts() / factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cycling")
    }

    #[test]
    fn fixture_age_equivalent() {
        let text = fs::read_to_string(fixtures_dir().join("age.csv")).unwrap();
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }
            let c: Vec<_> = line.split(',').collect();
            let ftp = Ftp::new(c[0].parse().unwrap()).unwrap();
            let age: u16 = c[1].parse().unwrap();
            let ref_age: u16 = c[2].parse().unwrap();
            let expected: f64 = c[3].parse().unwrap();
            let eq = age_equivalent_ftp(ftp, age, ref_age).unwrap();
            assert!(
                (eq.watts() - expected).abs() < 0.5,
                "row {i}: got {} want {expected}",
                eq.watts()
            );
        }
    }

    #[test]
    fn plateau_and_range() {
        assert!((age_factor(30, 35).unwrap() - 1.0).abs() < 1e-12);
        assert!((age_factor(55, 35).unwrap() - 0.9).abs() < 1e-12);
        assert_eq!(
            age_factor(14, 35),
            Err(Error::AgeOutOfRange { min: 15, max: 90 })
        );
        assert_eq!(
            age_factor(55, 91),
            Err(Error::AgeOutOfRange { min: 15, max: 90 })
        );
    }
}
