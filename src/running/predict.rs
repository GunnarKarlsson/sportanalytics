//! Race-time prediction.
//!
//! * [`PredictionModel::DanielsVdot`] — invert Daniels–Gilbert (recommended).
//! * [`PredictionModel::Riegel`] — `T2 = T1 * (D2/D1)^1.06` (Pete Riegel, 1977/1981).
//! * [`PredictionModel::Cameron`] — David Cameron road-race fit.
//!
//! Age and gender are not used by these models. Pass them only if you later
//! age-adjust a predicted time with [`crate::running::age_equivalent`].

use super::time::format_hms;
use super::vo2::{time_from_vdot, vdot};
use super::{Distance, Gender, RaceTime, Vdot};

/// Which scaling model to use for [`predict_times`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictionModel {
    /// Invert the Daniels–Gilbert VDOT equations (recommended).
    DanielsVdot,
    /// Riegel power law with exponent 1.06.
    Riegel,
    /// Cameron (1996-ish) road-race fit.
    Cameron,
}

/// Predicted finish times at the five supported distances, in seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct PredictedTimes {
    /// Model that produced these times.
    pub model: PredictionModel,
    /// VDOT implied by the input race (always computed; used by Daniels).
    pub vdot: Vdot,
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
    pub fn seconds(&self, d: Distance) -> f64 {
        match d {
            Distance::ThreeK => self.three_k,
            Distance::FiveK => self.five_k,
            Distance::TenK => self.ten_k,
            Distance::HalfMarathon => self.half_marathon,
            Distance::Marathon => self.marathon,
        }
    }

    /// Predicted finish time formatted as `h:mm:ss` or `m:ss`.
    pub fn formatted(&self, d: Distance) -> String {
        format_hms(self.seconds(d))
    }
}

/// Daniels VDOT equivalents and Cameron-scaled times from the same race.
#[derive(Debug, Clone, PartialEq)]
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
/// `age` and `gender` are unused by Daniels, Riegel, and Cameron. They are
/// accepted so callers can pass athlete metadata without a separate code path.
pub fn predict_times(
    known: RaceTime,
    model: PredictionModel,
    _age: Option<u8>,
    _gender: Option<Gender>,
) -> PredictedTimes {
    let vd = vdot(known);
    let secs = |target: Distance| match model {
        PredictionModel::DanielsVdot => time_from_vdot(vd, target),
        PredictionModel::Riegel => riegel(known, target),
        PredictionModel::Cameron => cameron(known, target),
    };
    PredictedTimes {
        model,
        vdot: vd,
        three_k: secs(Distance::ThreeK),
        five_k: secs(Distance::FiveK),
        ten_k: secs(Distance::TenK),
        half_marathon: secs(Distance::HalfMarathon),
        marathon: secs(Distance::Marathon),
    }
}

/// Daniels VDOT equivalents and Cameron-scaled times from the same race.
pub fn predict_daniels_and_cameron(known: RaceTime) -> DualPredictedTimes {
    DualPredictedTimes {
        vdot: vdot(known),
        daniels: predict_times(known, PredictionModel::DanielsVdot, None, None),
        cameron: predict_times(known, PredictionModel::Cameron, None, None),
    }
}

/// Riegel: `T2 = T1 × (D2 / D1)^k` with `k = 1.06`.
pub fn riegel(known: RaceTime, target: Distance) -> f64 {
    riegel_with_exponent(known, target, 1.06)
}

/// Riegel power law with a caller-supplied exponent.
pub fn riegel_with_exponent(known: RaceTime, target: Distance, k: f64) -> f64 {
    let ratio = target.meters() / known.distance().meters();
    known.seconds() * ratio.powf(k)
}

/// Cameron (1996-ish): `T2 = T1 × (D2/D1) × f(D1)/f(D2)`
/// with `f(x) = 13.49681 − 0.000030363 x + 835.7114 / x^0.7905`, `x` in metres.
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
        let pred = predict_times(five, PredictionModel::DanielsVdot, None, None);
        assert_eq!(pred.model, PredictionModel::DanielsVdot);
        assert!((pred.seconds(Distance::FiveK) - five.seconds()).abs() < 0.5);
        assert_eq!(pred.formatted(Distance::FiveK), "20:00");
    }

    #[test]
    fn dual_matches_calling_each_model() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let both = predict_daniels_and_cameron(five);
        let daniels = predict_times(five, PredictionModel::DanielsVdot, None, None);
        let cameron = predict_times(five, PredictionModel::Cameron, None, None);
        assert_eq!(both.vdot, daniels.vdot);
        assert_eq!(both.daniels, daniels);
        assert_eq!(both.cameron, cameron);
        assert_eq!(both.daniels.vdot, both.cameron.vdot);
    }

    #[test]
    fn unused_age_gender_do_not_change_prediction() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let a = predict_times(five, PredictionModel::Riegel, None, None);
        let b = predict_times(
            five,
            PredictionModel::Riegel,
            Some(42),
            Some(Gender::Female),
        );
        assert_eq!(a, b);
    }
}
