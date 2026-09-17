//! Field estimate of cycling VO2max (Hawley & Noakes).
//!
//! `VO2 (ml·kg⁻¹·min⁻¹) = 10.8 × (MAP / m) + 7`
//!
//! This is a **field estimate** from maximal aerobic power, not laboratory gas
//! analysis. If only FTP is known, MAP ≈ `1.20 × FTP` (extra error).

use std::fmt;

use super::effort::{Mass, Power};
use super::ftp::Ftp;
use crate::Error;

/// Estimated VO2max in ml·kg⁻¹·min⁻¹ from a field power test.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct EstimatedVo2(f64);

impl EstimatedVo2 {
    /// Numeric value in ml/kg/min.
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for EstimatedVo2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} ml/kg/min", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for EstimatedVo2 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for EstimatedVo2 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = <f64 as serde::Deserialize>::deserialize(deserializer)?;
        if v.is_finite() && v > 0.0 {
            Ok(EstimatedVo2(v))
        } else {
            Err(serde::de::Error::custom(
                "estimated VO2 must be a positive finite value",
            ))
        }
    }
}

/// Hawley–Noakes slope coefficient (`10.8`).
pub const HAWLEY_NOAKES_A: f64 = 10.8;
/// Hawley–Noakes intercept (`7.0`).
pub const HAWLEY_NOAKES_B: f64 = 7.0;
/// Approximate MAP from FTP: `MAP ≈ FTP_TO_MAP × FTP`.
pub const FTP_TO_MAP: f64 = 1.20;

/// Estimated VO2max from MAP and body mass (Hawley & Noakes).
///
/// ```
/// use sportanalytics::cycling::{estimated_vo2max, Mass, Power};
///
/// let vo2 = estimated_vo2max(Power::new(343.0).unwrap(), Mass::from_kg(70.0).unwrap());
/// assert!((vo2.value() - 59.91).abs() < 0.05);
/// ```
pub fn estimated_vo2max(map: Power, mass: Mass) -> EstimatedVo2 {
    EstimatedVo2(HAWLEY_NOAKES_A * (map.watts() / mass.kg()) + HAWLEY_NOAKES_B)
}

/// Estimated VO2max from FTP, using `MAP ≈ 1.20 × FTP`.
pub fn estimated_vo2max_from_ftp(ftp: Ftp, mass: Mass) -> EstimatedVo2 {
    let map = Power::new(FTP_TO_MAP * ftp.watts()).expect("FTP implies positive MAP");
    estimated_vo2max(map, mass)
}

/// Invert Hawley–Noakes: MAP from a target VO2 (ml/kg/min) and mass.
pub fn map_from_vo2(vo2_ml_kg_min: f64, mass: Mass) -> Result<Power, Error> {
    if !vo2_ml_kg_min.is_finite() || vo2_ml_kg_min <= HAWLEY_NOAKES_B {
        return Err(Error::InvalidPower);
    }
    let map_w = (vo2_ml_kg_min - HAWLEY_NOAKES_B) / HAWLEY_NOAKES_A * mass.kg();
    Power::new(map_w)
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
    fn fixture_hawley_noakes() {
        let text = fs::read_to_string(fixtures_dir().join("vo2.csv")).unwrap();
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }
            let c: Vec<_> = line.split(',').collect();
            let mass = Mass::from_kg(c[0].parse().unwrap()).unwrap();
            let map = Power::new(c[1].parse().unwrap()).unwrap();
            let expected: f64 = c[2].parse().unwrap();
            let vo2 = estimated_vo2max(map, mass);
            assert!(
                (vo2.value() - expected).abs() < 0.05,
                "row {i}: got {} want {expected}",
                vo2.value()
            );
            let back = map_from_vo2(vo2.value(), mass).unwrap();
            assert!((back.watts() - map.watts()).abs() < 0.05);
        }
    }

    #[test]
    fn from_ftp_uses_map_factor() {
        let ftp = Ftp::new(250.0).unwrap();
        let mass = Mass::from_kg(70.0).unwrap();
        let from_ftp = estimated_vo2max_from_ftp(ftp, mass);
        let from_map = estimated_vo2max(Power::new(300.0).unwrap(), mass);
        assert!((from_ftp.value() - from_map.value()).abs() < 1e-12);
    }
}
