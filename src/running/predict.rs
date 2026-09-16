//! Race-time prediction.
//!
//! * [`PredictionModel::DanielsVdot`] — invert Daniels–Gilbert (recommended).
//! * [`PredictionModel::Riegel`] — `T2 = T1 * (D2/D1)^1.06` (Pete Riegel, 1977/1981).
//! * [`PredictionModel::Cameron`] — David Cameron road-race fit.
//!
//! Age and gender are not inputs to these models. Age-adjust a predicted time
//! afterwards with [`crate::running::age_equivalent`].

use super::time::format_hms;
use super::vo2::{time_from_vdot, vdot};
use super::{Distance, RaceTime, Vdot};
use crate::Error;
use std::fmt;

/// Which scaling model to use for [`predict_times`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PredictionModel {
    /// Invert the Daniels–Gilbert VDOT equations (recommended).
    DanielsVdot,
    /// Riegel power law with exponent 1.06.
    Riegel,
    /// Cameron (1996-ish) road-race fit.
    Cameron,
}

/// Predicted finish times at the five named distances, in seconds.
///
/// [`Self::seconds`] also works for [`Distance::Custom`] by re-running the same
/// model from [`Self::source`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PredictedTimes {
    /// Model that produced these times.
    pub model: PredictionModel,
    /// VDOT implied by the input race (always computed; used by Daniels).
    pub vdot: Vdot,
    /// Race that was scaled.
    pub source: RaceTime,
    /// Predicted 3K time in seconds.
    pub three_k: f64,
    /// Predicted 5K time in seconds.
    pub five_k: f64,
    /// Predicted 10K time in seconds.
    pub ten_k: f64,
    /// Predicted half-marathon time in seconds.
    pub half_marathon: f64,
    /// Predicted marathon time in seconds.
    pub marathon: f64,
}

impl PredictedTimes {
    /// Predicted finish time in seconds for `d`.
    ///
    /// Named distances use the values computed by [`predict_times`]. Custom
    /// distances re-run the model; Daniels inversion can return
    /// [`Error::UnsolvableTime`].
    ///
    /// ```
    /// use sportanalytics::running::{predict_times, Distance, PredictionModel, RaceTime};
    ///
    /// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
    /// let pred = predict_times(five, PredictionModel::DanielsVdot).unwrap();
    /// assert!((pred.seconds(Distance::FiveK).unwrap() - 1200.0).abs() < 0.5);
    /// ```
    pub fn seconds(&self, d: Distance) -> Result<f64, Error> {
        Ok(match d {
            Distance::ThreeK => self.three_k,
            Distance::FiveK => self.five_k,
            Distance::TenK => self.ten_k,
            Distance::HalfMarathon => self.half_marathon,
            Distance::Marathon => self.marathon,
            Distance::Custom { .. } => match self.model {
                PredictionModel::DanielsVdot => time_from_vdot(self.vdot, d)?,
                PredictionModel::Riegel => riegel(self.source, d),
                PredictionModel::Cameron => cameron(self.source, d),
            },
        })
    }

    /// Predicted finish time formatted as `h:mm:ss` or `m:ss`.
    pub fn formatted(&self, d: Distance) -> Result<String, Error> {
        Ok(format_hms(self.seconds(d)?))
    }
}

impl fmt::Display for PredictedTimes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} VDOT {}  3K {}  5K {}  10K {}  HM {}  FM {}",
            self.model,
            self.vdot,
            format_hms(self.three_k),
            format_hms(self.five_k),
            format_hms(self.ten_k),
            format_hms(self.half_marathon),
            format_hms(self.marathon)
        )
    }
}

/// Daniels VDOT equivalents and Cameron-scaled times from the same race.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DualPredictedTimes {
    /// VDOT implied by the input race.
    pub vdot: Vdot,
    /// Predictions from [`PredictionModel::DanielsVdot`].
    pub daniels: PredictedTimes,
    /// Predictions from [`PredictionModel::Cameron`].
    pub cameron: PredictedTimes,
}

/// Predict 3K / 5K / 10K / HM / FM from a single known race.
///
/// These models are distance/time only. For an age-adjusted equivalent, take a
/// predicted time and pass it to [`crate::running::age_equivalent`].
///
/// [`PredictionModel::DanielsVdot`] returns [`Error::UnsolvableTime`] when any
/// named distance has no root in the 2–12 min/km inversion bracket.
///
/// ```
/// use sportanalytics::running::{predict_times, Distance, PredictionModel, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
/// let pred = predict_times(five, PredictionModel::DanielsVdot).unwrap();
/// assert_eq!(pred.formatted(Distance::FiveK).unwrap(), "20:00");
/// ```
pub fn predict_times(known: RaceTime, model: PredictionModel) -> Result<PredictedTimes, Error> {
    let vd = vdot(known);
    let secs = |target: Distance| match model {
        PredictionModel::DanielsVdot => time_from_vdot(vd, target),
        PredictionModel::Riegel => Ok(riegel(known, target)),
        PredictionModel::Cameron => Ok(cameron(known, target)),
    };
    Ok(PredictedTimes {
        model,
        vdot: vd,
        source: known,
        three_k: secs(Distance::ThreeK)?,
        five_k: secs(Distance::FiveK)?,
        ten_k: secs(Distance::TenK)?,
        half_marathon: secs(Distance::HalfMarathon)?,
        marathon: secs(Distance::Marathon)?,
    })
}

/// Daniels VDOT equivalents and Cameron-scaled times from the same race.
///
/// ```
/// use sportanalytics::running::{predict_daniels_and_cameron, Distance, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
/// let both = predict_daniels_and_cameron(five).unwrap();
/// assert_eq!(both.daniels.vdot, both.cameron.vdot);
/// ```
pub fn predict_daniels_and_cameron(known: RaceTime) -> Result<DualPredictedTimes, Error> {
    Ok(DualPredictedTimes {
        vdot: vdot(known),
        daniels: predict_times(known, PredictionModel::DanielsVdot)?,
        cameron: predict_times(known, PredictionModel::Cameron)?,
    })
}

/// Riegel: `T2 = T1 × (D2 / D1)^k` with `k = 1.06`.
///
/// ```
/// use sportanalytics::running::{riegel, Distance, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 25, 0).unwrap();
/// let ten = riegel(five, Distance::TenK);
/// assert!((ten - 3127.0).abs() < 5.0);
/// ```
pub fn riegel(known: RaceTime, target: Distance) -> f64 {
    riegel_with_exponent(known, target, 1.06)
}

/// Riegel power law with a caller-supplied exponent.
///
/// ```
/// use sportanalytics::running::{riegel_with_exponent, Distance, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
/// let ten = riegel_with_exponent(five, Distance::TenK, 1.0);
/// assert!((ten - 2400.0).abs() < 1e-9);
/// ```
pub fn riegel_with_exponent(known: RaceTime, target: Distance, k: f64) -> f64 {
    let ratio = target.meters() / known.distance().meters();
    known.seconds() * ratio.powf(k)
}

/// Cameron (1996-ish): `T2 = T1 × (D2/D1) × f(D1)/f(D2)`
/// with `f(x) = 13.49681 − 0.000030363 x + 835.7114 / x^0.7905`, `x` in metres.
///
/// ```
/// use sportanalytics::running::{cameron, Distance, RaceTime};
///
/// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
/// assert!((cameron(five, Distance::FiveK) - five.seconds()).abs() < 1e-9);
/// ```
pub fn cameron(known: RaceTime, target: Distance) -> f64 {
    let d1 = known.distance().meters();
    let d2 = target.meters();
    known.seconds() * (d2 / d1) * (cameron_f(d1) / cameron_f(d2))
}

fn cameron_f(meters: f64) -> f64 {
    13.49681 - 0.000030363 * meters + 835.7114 / meters.powf(0.7905)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn riegel_doubling_adds_fatigue() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 25, 0).unwrap();
        let ten = riegel(five, Distance::TenK);
        // 25:00 * 2^1.06 ≈ 52:07
        assert!((ten - 3127.0).abs() < 5.0, "got {ten}");
    }

    #[test]
    fn riegel_same_distance_is_identity() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let back = riegel(five, Distance::FiveK);
        assert!((back - five.seconds()).abs() < 1e-9);
    }

    #[test]
    fn riegel_exponent_one_scales_linearly() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let ten = riegel_with_exponent(five, Distance::TenK, 1.0);
        assert!((ten - 2400.0).abs() < 1e-9);
    }

    #[test]
    fn cameron_same_distance_is_identity() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let back = cameron(five, Distance::FiveK);
        assert!((back - five.seconds()).abs() < 1e-9);
    }

    #[test]
    fn cameron_5k_20_min_to_10k() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let ten = cameron(five, Distance::TenK);
        // 1200 × 2 × f(5000)/f(10000) with Cameron's f(x)
        assert!((ten - 2499.661372648436).abs() < 1e-9, "got {ten}");
    }

    #[test]
    fn cameron_longer_distance_is_slower() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let ten = cameron(five, Distance::TenK);
        let hm = cameron(five, Distance::HalfMarathon);
        assert!(ten > five.seconds() * 2.0);
        assert!(hm > ten);
    }

    #[test]
    fn daniels_same_distance_roundtrips() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let pred = predict_times(five, PredictionModel::DanielsVdot).unwrap();
        assert_eq!(pred.model, PredictionModel::DanielsVdot);
        assert!((pred.seconds(Distance::FiveK).unwrap() - five.seconds()).abs() < 0.5);
        assert_eq!(pred.formatted(Distance::FiveK).unwrap(), "20:00");
    }

    #[test]
    fn dual_matches_calling_each_model() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let both = predict_daniels_and_cameron(five).unwrap();
        let daniels = predict_times(five, PredictionModel::DanielsVdot).unwrap();
        let cameron = predict_times(five, PredictionModel::Cameron).unwrap();
        assert_eq!(both.vdot, daniels.vdot);
        assert_eq!(both.daniels, daniels);
        assert_eq!(both.cameron, cameron);
        assert_eq!(both.daniels.vdot, both.cameron.vdot);
    }

    #[test]
    fn riegel_and_daniels_are_distinct_models() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let riegel = predict_times(five, PredictionModel::Riegel).unwrap();
        let daniels = predict_times(five, PredictionModel::DanielsVdot).unwrap();
        assert_eq!(riegel.model, PredictionModel::Riegel);
        assert_eq!(daniels.model, PredictionModel::DanielsVdot);
        assert!(
            (riegel.seconds(Distance::Marathon).unwrap()
                - daniels.seconds(Distance::Marathon).unwrap())
            .abs()
                > 1.0
        );
    }

    #[test]
    fn custom_distance_scales_with_riegel() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let eight = Distance::custom(8_000.0, "8K").unwrap();
        let pred = predict_times(five, PredictionModel::Riegel).unwrap();
        let expected = riegel(five, eight);
        assert!((pred.seconds(eight).unwrap() - expected).abs() < 1e-9);
        assert_eq!(pred.source, five);
    }

    #[test]
    fn daniels_rejects_race_outside_solver_bracket() {
        let slow = RaceTime::from_hms(Distance::FiveK, 1, 0, 0).unwrap();
        assert_eq!(
            predict_times(slow, PredictionModel::DanielsVdot),
            Err(Error::UnsolvableTime)
        );
        assert!(predict_times(slow, PredictionModel::Riegel).is_ok());
    }

    #[test]
    fn predicted_times_display_includes_distances() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let s = predict_times(five, PredictionModel::DanielsVdot)
            .unwrap()
            .to_string();
        assert!(s.contains("5K"));
        assert!(s.contains("HM"));
        assert!(s.contains("VDOT"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_predicted_times_roundtrip() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let pred = predict_times(five, PredictionModel::DanielsVdot).unwrap();
        let back: PredictedTimes =
            serde_json::from_str(&serde_json::to_string(&pred).unwrap()).unwrap();
        assert_eq!(back.model, pred.model);
        assert_eq!(back.source, pred.source);
        assert!((back.vdot.value() - pred.vdot.value()).abs() < 1e-12);
        for d in Distance::all() {
            assert!((back.seconds(d).unwrap() - pred.seconds(d).unwrap()).abs() < 1e-6);
        }
    }
}
