//! Field estimates of cycling VO2max from maximal aerobic power (MAP).
//!
//! **ACSM (relative):** `VO2 (ml·kg⁻¹·min⁻¹) = 10.8 × (MAP / m) + 7`
//!
//! **Hawley & Noakes 1992 (absolute):** `VO2 (L/min) = 0.01141 × Wpeak + 0.435`
//! (convert to relative by ×1000 / mass_kg).
//!
//! These are **field estimates** from MAP / Wpeak, not laboratory gas analysis.
//! If only FTP is known, MAP ≈ `1.20 × FTP` (extra error). Do not label the
//! ACSM relative equation as Hawley–Noakes.

use std::fmt;

use super::effort::{Mass, Power};
use super::ftp::Ftp;
use super::Error;

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

/// ACSM relative VO2 slope (`10.8`).
pub const ACSM_A: f64 = 10.8;
/// ACSM relative VO2 intercept (`7.0`).
pub const ACSM_B: f64 = 7.0;
/// Hawley–Noakes 1992 absolute VO2 slope (`0.01141` L/min per watt).
pub const HAWLEY_NOAKES_A: f64 = 0.01141;
/// Hawley–Noakes 1992 absolute VO2 intercept (`0.435` L/min).
pub const HAWLEY_NOAKES_B: f64 = 0.435;
/// Approximate MAP from FTP: `MAP ≈ FTP_TO_MAP × FTP`.
pub const FTP_TO_MAP: f64 = 1.20;

/// Estimated relative VO2max from MAP and body mass (ACSM cycling equation).
///
/// ```
/// use sportanalytics::cycling::{estimated_vo2max, Mass, Power};
///
/// let vo2 = estimated_vo2max(Power::new(343.0).unwrap(), Mass::from_kg(70.0).unwrap());
/// assert!((vo2.value() - 59.91).abs() < 0.05);
/// ```
pub fn estimated_vo2max(map: Power, mass: Mass) -> EstimatedVo2 {
    EstimatedVo2(ACSM_A * (map.watts() / mass.kg()) + ACSM_B)
}

/// Estimated relative VO2max from MAP via Hawley & Noakes 1992 (absolute → relative).
///
/// `VO2_rel = 1000 × (0.01141 × Wpeak + 0.435) / mass_kg`.
pub fn estimated_vo2max_hawley_noakes(map: Power, mass: Mass) -> EstimatedVo2 {
    let l_min = HAWLEY_NOAKES_A * map.watts() + HAWLEY_NOAKES_B;
    EstimatedVo2(l_min * 1000.0 / mass.kg())
}

/// Estimated VO2max from FTP, using `MAP ≈ 1.20 × FTP` and the ACSM equation.
pub fn estimated_vo2max_from_ftp(ftp: Ftp, mass: Mass) -> EstimatedVo2 {
    let map = Power::new(FTP_TO_MAP * ftp.watts()).expect("FTP implies positive MAP");
    estimated_vo2max(map, mass)
}

/// Invert ACSM: MAP from a target relative VO2 (ml/kg/min) and mass.
pub fn map_from_vo2(vo2_ml_kg_min: f64, mass: Mass) -> Result<Power, Error> {
    if !vo2_ml_kg_min.is_finite() || vo2_ml_kg_min <= ACSM_B {
        return Err(Error::InvalidPower);
    }
    let map_w = (vo2_ml_kg_min - ACSM_B) / ACSM_A * mass.kg();
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
    fn fixture_vo2_equations() {
        let text = fs::read_to_string(fixtures_dir().join("vo2.csv")).unwrap();
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }
            let c: Vec<_> = line.split(',').collect();
            let equation = c[0];
            let mass = Mass::from_kg(c[1].parse().unwrap()).unwrap();
            let map = Power::new(c[2].parse().unwrap()).unwrap();
            let expected: f64 = c[3].parse().unwrap();
            let vo2 = match equation {
                "acsm" => estimated_vo2max(map, mass),
                "hawley" => estimated_vo2max_hawley_noakes(map, mass),
                other => panic!("unknown equation {other}"),
            };
            assert!(
                (vo2.value() - expected).abs() < 0.05,
                "row {i} ({equation}): got {} want {expected}",
                vo2.value()
            );
            if equation == "acsm" {
                let back = map_from_vo2(vo2.value(), mass).unwrap();
                assert!((back.watts() - map.watts()).abs() < 0.05);
            }
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
