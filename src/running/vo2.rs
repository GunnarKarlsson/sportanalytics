//! Jack Daniels & Jimmy Gilbert (1979) VDOT / effective VO2max.
//!
//! ```text
//! VO2(v)     = -4.60 + 0.182258 v + 0.000104 v²     (v in m/min)
//! %VO2max(t) = 0.8 + 0.1894393 e^(-0.012778 t)
//!                  + 0.2989558 e^(-0.1932605 t)     (t in minutes)
//! VDOT       = VO2(v) / %VO2max(t)
//! ```
//!
//! VDOT is *effective* VO2max (running economy included), not a laboratory
//! VO2max test.

use std::fmt;

use super::{Distance, RaceTime};
use crate::Error;

/// Newtype for a Daniels VDOT (effective VO2max) value in ml/kg/min.
///
/// Inner arithmetic stays on [`f64`]. Use this wrapper at API edges so a VDOT
/// is not confused with seconds, metres/min, or a %VO2max fraction.
///
/// [`Self::new`] accepts any positive finite value. [`time_from_vdot`] only
/// solves finish times between 2 and 12 min/km.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vdot(f64);

impl Vdot {
    /// Construct a VDOT from a positive finite value.
    pub fn new(value: f64) -> Result<Self, Error> {
        if value.is_finite() && value > 0.0 {
            Ok(Self(value))
        } else {
            Err(Error::InvalidVdot)
        }
    }

    /// The numeric VDOT in ml/kg/min.
    pub const fn value(self) -> f64 {
        self.0
    }

    /// VDOT implied by a single race result.
    pub fn from_race(race: RaceTime) -> Self {
        Self::from_raw(vdot_value(race))
    }

    pub(crate) fn from_raw(value: f64) -> Self {
        Self(value)
    }
}

impl fmt::Display for Vdot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Vdot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Vdot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <f64 as serde::Deserialize>::deserialize(deserializer)?;
        Vdot::new(value).map_err(serde::de::Error::custom)
    }
}

/// Combined VDOT estimate from one or more races.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vo2Estimate {
    /// Per-race VDOT values, same order as the input slice.
    pub per_race: Vec<(Distance, Vdot)>,
    /// Mean VDOT across the supplied races.
    pub mean: Vdot,
    /// Best (highest) VDOT — usually the most recent / best-trained distance.
    pub best: Vdot,
}

/// Daniels oxygen cost of running at velocity `v` (m/min). ml/kg/min.
pub fn oxygen_cost(v_m_per_min: f64) -> f64 {
    -4.60 + 0.182258 * v_m_per_min + 0.000104 * v_m_per_min * v_m_per_min
}

/// Sustainable fraction of VO2max for a race lasting `t` minutes.
pub fn percent_vo2max(t_min: f64) -> f64 {
    0.8 + 0.1894393 * (-0.012778 * t_min).exp() + 0.2989558 * (-0.1932605 * t_min).exp()
}

/// Invert the oxygen-cost curve: velocity (m/min) that costs `vo2` ml/kg/min.
pub fn velocity_from_vo2(vo2: f64) -> f64 {
    // 0.000104 v² + 0.182258 v - (vo2 + 4.60) = 0
    let a = 0.000104;
    let b = 0.182258;
    let c = -(vo2 + 4.60);
    let disc = (b * b - 4.0 * a * c).max(0.0);
    (-b + disc.sqrt()) / (2.0 * a)
}

fn vdot_value(race: RaceTime) -> f64 {
    let vo2 = oxygen_cost(race.velocity_m_per_min());
    let frac = percent_vo2max(race.minutes());
    vo2 / frac
}

/// Effective VO2max (VDOT) from one race.
pub fn vdot(race: RaceTime) -> Vdot {
    Vdot::from_race(race)
}

/// Combine one or more race times into a VO2max / VDOT estimate.
pub fn vo2max_from_races(races: &[RaceTime]) -> Result<Vo2Estimate, Error> {
    let per_race: Vec<(Distance, Vdot)> = races.iter().map(|r| (r.distance(), vdot(*r))).collect();
    let best = per_race
        .iter()
        .map(|(_, v)| *v)
        .max_by(|a, b| a.value().total_cmp(&b.value()))
        .ok_or(Error::EmptyRaces)?;
    let mean = Vdot::from_raw(
        per_race.iter().map(|(_, v)| v.value()).sum::<f64>() / per_race.len() as f64,
    );
    Ok(Vo2Estimate {
        per_race,
        mean,
        best,
    })
}

fn implied_vdot(meters: f64, time_secs: f64) -> f64 {
    let t_min = time_secs / 60.0;
    oxygen_cost(meters / t_min) / percent_vo2max(t_min)
}

/// Predicted finish time (seconds) at `distance` for a given VDOT.
///
/// Solves `VDOT * %VO2max(t) = VO2(distance / t)` by bisection over finish
/// times from **2 min/km** (elite) to **12 min/km** (very slow). Returns
/// [`Error::UnsolvableTime`] when no root lies in that bracket — extreme
/// VDOTs are not clamped to the endpoints.
pub fn time_from_vdot(vdot: Vdot, distance: Distance) -> Result<f64, Error> {
    let meters = distance.meters();
    let vd = vdot.value();
    let mut lo = meters / 1000.0 * 2.0 * 60.0;
    let mut hi = meters / 1000.0 * 12.0 * 60.0;
    if implied_vdot(meters, lo) < vd || implied_vdot(meters, hi) > vd {
        return Err(Error::UnsolvableTime);
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let implied = implied_vdot(meters, mid);
        if implied > vd {
            // too fast for this VDOT → need a slower (larger) time
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_k_20_min_is_about_vdot_50() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let v = vdot(race).value();
        assert!((v - 49.8).abs() < 0.4, "got {v}");
    }

    #[test]
    fn ten_k_45_min_is_about_vdot_45() {
        let race = RaceTime::from_hms(Distance::TenK, 0, 45, 0).unwrap();
        let v = vdot(race).value();
        assert!((v - 45.3).abs() < 0.4, "got {v}");
    }

    #[test]
    fn from_race_matches_vdot_fn() {
        let race = RaceTime::from_hms(Distance::HalfMarathon, 1, 30, 0).unwrap();
        assert_eq!(Vdot::from_race(race), vdot(race));
    }

    #[test]
    fn vdot_new_rejects_invalid_values() {
        assert_eq!(Vdot::new(0.0), Err(Error::InvalidVdot));
        assert_eq!(Vdot::new(-1.0), Err(Error::InvalidVdot));
        assert_eq!(Vdot::new(f64::NAN), Err(Error::InvalidVdot));
        assert_eq!(Vdot::new(f64::INFINITY), Err(Error::InvalidVdot));
        assert_eq!(Vdot::new(f64::NEG_INFINITY), Err(Error::InvalidVdot));
        assert!(Vdot::new(50.0).is_ok());
    }

    #[test]
    fn empty_races_are_rejected() {
        assert_eq!(vo2max_from_races(&[]), Err(Error::EmptyRaces));
    }

    #[test]
    fn vo2max_from_races_mean_and_best() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let ten = RaceTime::from_hms(Distance::TenK, 0, 45, 0).unwrap();
        let est = vo2max_from_races(&[five, ten]).unwrap();
        assert_eq!(est.per_race.len(), 2);
        let expected_mean = (est.per_race[0].1.value() + est.per_race[1].1.value()) / 2.0;
        assert!((est.mean.value() - expected_mean).abs() < 1e-12);
        let expected_best = est.per_race[0].1.value().max(est.per_race[1].1.value());
        assert!((est.best.value() - expected_best).abs() < 1e-12);
    }

    #[test]
    fn time_from_vdot_roundtrips_5k() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let vd = vdot(race);
        let predicted = time_from_vdot(vd, Distance::FiveK).unwrap();
        assert!(
            (predicted - race.seconds()).abs() < 0.5,
            "got {predicted}, want {}",
            race.seconds()
        );
    }

    #[test]
    fn time_from_vdot_rejects_outside_pace_bracket() {
        let five = Distance::FiveK;
        assert_eq!(
            time_from_vdot(Vdot::new(1.0).unwrap(), five),
            Err(Error::UnsolvableTime)
        );
        assert_eq!(
            time_from_vdot(Vdot::new(200.0).unwrap(), five),
            Err(Error::UnsolvableTime)
        );
        assert!(time_from_vdot(Vdot::new(50.0).unwrap(), five).is_ok());
    }

    #[test]
    fn velocity_from_vo2_inverts_oxygen_cost() {
        let v = 250.0;
        let vo2 = oxygen_cost(v);
        let back = velocity_from_vo2(vo2);
        assert!((back - v).abs() < 1e-6, "got {back}");
    }

    #[test]
    fn percent_vo2max_decreases_with_duration() {
        assert!(percent_vo2max(15.0) > percent_vo2max(30.0));
        assert!(percent_vo2max(30.0) > percent_vo2max(120.0));
        assert!(percent_vo2max(10.0) > 0.8);
    }

    #[test]
    fn vdot_display_one_decimal() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let s = vdot(race).to_string();
        assert!(s.contains('.'), "got {s}");
        assert_eq!(s.chars().filter(|c| *c == '.').count(), 1);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_vdot_roundtrip_and_rejects_invalid() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let vd = vdot(race);
        let back: Vdot = serde_json::from_str(&serde_json::to_string(&vd).unwrap()).unwrap();
        assert_eq!(back.value(), vd.value());
        let est = vo2max_from_races(&[race]).unwrap();
        assert_eq!(
            serde_json::from_str::<Vo2Estimate>(&serde_json::to_string(&est).unwrap()).unwrap(),
            est
        );
        assert!(serde_json::from_str::<Vdot>("0").is_err());
        assert!(serde_json::from_str::<Vdot>("-1").is_err());
    }
}
