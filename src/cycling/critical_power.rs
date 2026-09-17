//! Two-parameter critical power (CP) and W′.
//!
//! ```text
//! P(t) = CP + W′/t
//! t    = W′/(P − CP)   (P > CP)
//! ```
//!
//! Fit with OLS on `(1/t, P)`: intercept = CP, slope = W′.
//! The model is valid roughly **2–30 min** for fit inputs; this crate caps
//! predictions at **60 min**. Hour power extrapolated from a short + medium
//! pair (e.g. 5 min + 20 min) is **biased high** — treat that output as model
//! math, not “true FTP”. Do not use for 5 s sprints or multi-hour rides.

use std::time::Duration;

use super::effort::{Effort, Power, Work};
use super::Error;

/// Two-parameter critical-power model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CriticalPower {
    /// Critical power (asymptotic sustainable power) in watts.
    pub cp: Power,
    /// Anaerobic work capacity W′ in joules.
    pub w_prime: Work,
}

#[cfg(feature = "serde")]
impl serde::Serialize for CriticalPower {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CriticalPower", 2)?;
        state.serialize_field("cp", &self.cp)?;
        state.serialize_field("w_prime", &self.w_prime)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CriticalPower {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Helper {
            cp: Power,
            w_prime: Work,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(CriticalPower {
            cp: h.cp,
            w_prime: h.w_prime,
        })
    }
}

/// Result of fitting [`CriticalPower`] to maximal efforts.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CpFit {
    /// Fitted CP + W′ model.
    pub model: CriticalPower,
    /// Coefficient of determination for the linear fit on \((1/t, P)\).
    pub r_squared: f64,
    /// Number of efforts used.
    pub n: usize,
}

const FIT_DURATION_LO: f64 = 120.0;
const FIT_DURATION_HI: f64 = 1800.0;
const PREDICT_DURATION_LO: f64 = 120.0;
const PREDICT_DURATION_HI: f64 = 3600.0;

fn check_fit_duration(seconds: f64) -> Result<(), Error> {
    if (FIT_DURATION_LO..=FIT_DURATION_HI).contains(&seconds) {
        Ok(())
    } else {
        Err(Error::DurationOutOfModelRange)
    }
}

fn check_predict_duration(seconds: f64) -> Result<(), Error> {
    if (PREDICT_DURATION_LO..=PREDICT_DURATION_HI).contains(&seconds) {
        Ok(())
    } else {
        Err(Error::DurationOutOfModelRange)
    }
}

/// Fit two-parameter CP + W′ from ≥ 2 maximal efforts (OLS on \(1/t\) vs \(P\)).
///
/// Each effort duration must be 120–1800 s. Three or four efforts are preferred.
///
/// ```
/// use sportanalytics::cycling::{critical_power, Effort};
///
/// let five = Effort::from_watts_secs(310.0, 300.0).unwrap();
/// let twenty = Effort::from_watts_secs(265.0, 1200.0).unwrap();
/// let fit = critical_power(&[five, twenty]).unwrap();
/// assert!((fit.model.cp.watts() - 250.0).abs() < 0.5);
/// assert!((fit.model.w_prime.joules() - 18_000.0).abs() < 50.0);
/// assert!(fit.r_squared > 0.999);
/// ```
pub fn critical_power(efforts: &[Effort]) -> Result<CpFit, Error> {
    if efforts.len() < 2 {
        return Err(Error::InsufficientEfforts);
    }

    let mut xs = Vec::with_capacity(efforts.len());
    let mut ys = Vec::with_capacity(efforts.len());
    for e in efforts {
        check_fit_duration(e.seconds())?;
        xs.push(1.0 / e.seconds());
        ys.push(e.power().watts());
    }

    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;

    let mut sxx = 0.0;
    let mut sxy = 0.0;
    let mut syy = 0.0;
    for i in 0..xs.len() {
        let dx = xs[i] - mean_x;
        let dy = ys[i] - mean_y;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }

    if sxx == 0.0 || !sxx.is_finite() {
        return Err(Error::UnsolvablePowerDuration);
    }

    let w_prime = sxy / sxx;
    let cp = mean_y - w_prime * mean_x;

    if !cp.is_finite() || cp <= 0.0 || !w_prime.is_finite() || w_prime <= 0.0 {
        return Err(Error::UnsolvablePowerDuration);
    }

    let r_squared = if syy == 0.0 {
        1.0
    } else {
        let ss_res: f64 = xs
            .iter()
            .zip(ys.iter())
            .map(|(x, y)| {
                let pred = cp + w_prime * x;
                let e = y - pred;
                e * e
            })
            .sum();
        (1.0 - ss_res / syy).clamp(0.0, 1.0)
    };

    Ok(CpFit {
        model: CriticalPower {
            cp: Power::new(cp)?,
            w_prime: Work::new(w_prime)?,
        },
        r_squared,
        n: efforts.len(),
    })
}

/// Predict mean power for `duration` from a fitted CP model: `P = CP + W'/t`.
///
/// `duration` must be 2–60 min. Values near the hour, especially when CP was
/// fitted from short efforts only, are model extrapolations biased high — not
/// laboratory FTP.
pub fn predict_power_from_cp(model: CriticalPower, duration: Duration) -> Result<Power, Error> {
    let t = duration.as_secs_f64();
    check_predict_duration(t)?;
    let p = model.cp.watts() + model.w_prime.joules() / t;
    Power::new(p).map_err(|_| Error::UnsolvablePowerDuration)
}

/// Predict duration at `power` from CP: `t = W' / (P − CP)` requiring `P > CP`.
pub fn predict_duration_from_cp(model: CriticalPower, power: Power) -> Result<Duration, Error> {
    let excess = power.watts() - model.cp.watts();
    if excess <= 0.0 {
        return Err(Error::UnsolvablePowerDuration);
    }
    let t = model.w_prime.joules() / excess;
    check_predict_duration(t)?;
    if !t.is_finite() || t <= 0.0 {
        return Err(Error::UnsolvablePowerDuration);
    }
    Ok(Duration::from_secs_f64(t))
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
    fn fixture_perfect_two_param() {
        let text = fs::read_to_string(fixtures_dir().join("cp.csv")).unwrap();
        let mut efforts = Vec::new();
        let mut expected_cp = 0.0;
        let mut expected_w = 0.0;
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }
            let c: Vec<_> = line.split(',').collect();
            // watts,seconds,cp,w_prime
            let watts: f64 = c[0].parse().unwrap();
            let seconds: f64 = c[1].parse().unwrap();
            expected_cp = c[2].parse().unwrap();
            expected_w = c[3].parse().unwrap();
            efforts.push(Effort::from_watts_secs(watts, seconds).unwrap());
        }
        let fit = critical_power(&efforts).unwrap();
        assert!((fit.model.cp.watts() - expected_cp).abs() < 0.5);
        assert!((fit.model.w_prime.joules() - expected_w).abs() < 50.0);
        assert!((fit.r_squared - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_one_effort_and_sprint() {
        let e = Effort::from_watts_secs(400.0, 300.0).unwrap();
        assert_eq!(critical_power(&[e]), Err(Error::InsufficientEfforts));
        let sprint = Effort::from_watts_secs(800.0, 30.0).unwrap();
        let twenty = Effort::from_watts_secs(265.0, 1200.0).unwrap();
        assert_eq!(
            critical_power(&[sprint, twenty]),
            Err(Error::DurationOutOfModelRange)
        );
    }

    #[test]
    fn predict_roundtrip() {
        let model = CriticalPower {
            cp: Power::new(250.0).unwrap(),
            w_prime: Work::new(18_000.0).unwrap(),
        };
        let p = predict_power_from_cp(model, Duration::from_secs(300)).unwrap();
        assert!((p.watts() - 310.0).abs() < 1e-9);
        let t = predict_duration_from_cp(model, p).unwrap();
        assert!((t.as_secs_f64() - 300.0).abs() < 1e-9);
        assert_eq!(
            predict_duration_from_cp(model, Power::new(250.0).unwrap()),
            Err(Error::UnsolvablePowerDuration)
        );
        assert_eq!(
            predict_duration_from_cp(model, Power::new(200.0).unwrap()),
            Err(Error::UnsolvablePowerDuration)
        );
    }

    #[test]
    fn hour_from_five_and_twenty_is_model_output_not_ftp() {
        // Perfect two-param pair: CP=250, W'=18000 → P(3600)=255 W.
        // Documented as CP-model output, not “true FTP”.
        let five = Effort::from_watts_secs(310.0, 300.0).unwrap();
        let twenty = Effort::from_watts_secs(265.0, 1200.0).unwrap();
        let fit = critical_power(&[five, twenty]).unwrap();
        let hour = predict_power_from_cp(fit.model, Duration::from_secs(3600)).unwrap();
        assert!((hour.watts() - 255.0).abs() < 0.5);
    }
}
