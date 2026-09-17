//! Power–duration prediction.
//!
//! * [`PredictionModel::CriticalPower`] — invert two-parameter CP + W′ (needs ≥ 2 efforts).
//! * [`PredictionModel::FtpPercent`] — single effort → FTP via protocol, then %FTP(t).
//! * [`PredictionModel::PowerRiegel`] — `P2 = P1 * (t1/t2)^k` with default `k = 0.07`.

use std::time::Duration;

use super::critical_power::{critical_power, predict_power_from_cp, CriticalPower};
use super::effort::{Effort, Power};
use super::ftp::{ftp_from_map, ftp_from_protocol, Ftp, FtpProtocol, MAP_TO_FTP};
use crate::Error;

/// Default power-side Riegel exponent (`k = 0.07`).
pub const RIEGEL_POWER_EXPONENT: f64 = 0.07;

/// Which model to use for [`predict_power`] / [`predict_duration`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PredictionModel {
    /// Two-parameter CP + W′ (requires ≥ 2 efforts via [`predict_power_from_efforts`]).
    CriticalPower,
    /// Scale a typical %FTP-by-duration curve from one effort.
    FtpPercent,
    /// Power Riegel: `P2 = P1 * (t1/t2)^k` with [`RIEGEL_POWER_EXPONENT`].
    PowerRiegel,
}

const RIEGEL_TARGET_LO: f64 = 120.0;
const RIEGEL_TARGET_HI: f64 = 6.0 * 3600.0;

/// Anchors for [`PredictionModel::FtpPercent`] (duration seconds → fraction of FTP).
const FTP_PCT_ANCHORS: [(f64, f64); 4] = [
    (5.0 * 60.0, 1.12),
    (20.0 * 60.0, 1.05),
    (60.0 * 60.0, 1.00),
    (180.0 * 60.0, 0.88),
];

fn ftp_percent_at(seconds: f64) -> Result<f64, Error> {
    let lo = FTP_PCT_ANCHORS[0].0;
    let hi = FTP_PCT_ANCHORS[FTP_PCT_ANCHORS.len() - 1].0;
    if seconds < lo || seconds > hi {
        return Err(Error::DurationOutOfModelRange);
    }
    for w in FTP_PCT_ANCHORS.windows(2) {
        let (t0, p0) = w[0];
        let (t1, p1) = w[1];
        if seconds >= t0 && seconds <= t1 {
            let log_t = seconds.ln();
            let log_t0 = t0.ln();
            let log_t1 = t1.ln();
            let u = (log_t - log_t0) / (log_t1 - log_t0);
            return Ok(p0 + u * (p1 - p0));
        }
    }
    Err(Error::DurationOutOfModelRange)
}

fn ftp_from_single_effort(effort: Effort) -> Result<Ftp, Error> {
    let t = effort.seconds();
    // Prefer exact protocol windows; otherwise nearest of 5 / 20 / 60 min heuristics.
    if (15.0 * 60.0..=25.0 * 60.0).contains(&t) {
        return ftp_from_protocol(effort, FtpProtocol::TwentyMin);
    }
    if (45.0 * 60.0..=75.0 * 60.0).contains(&t) {
        return ftp_from_protocol(effort, FtpProtocol::SixtyMin);
    }
    if (3.0 * 60.0..=8.0 * 60.0).contains(&t) {
        // ~5 min MAP
        return ftp_from_map(effort.power());
    }
    // Nearest of 5 / 20 / 60 min.
    let d5 = (t - 300.0).abs();
    let d20 = (t - 1200.0).abs();
    let d60 = (t - 3600.0).abs();
    if d5 <= d20 && d5 <= d60 {
        Ftp::new(effort.power().watts() * MAP_TO_FTP)
    } else if d20 <= d60 {
        Ftp::new(effort.power().watts() * 0.95)
    } else {
        Ftp::new(effort.power().watts())
    }
}

fn check_riegel_target(seconds: f64) -> Result<(), Error> {
    if (RIEGEL_TARGET_LO..=RIEGEL_TARGET_HI).contains(&seconds) {
        Ok(())
    } else {
        Err(Error::DurationOutOfModelRange)
    }
}

/// Power Riegel: `P2 = P1 * (t1/t2)^k`.
pub fn riegel_power(effort: Effort, target: Duration, k: f64) -> Result<Power, Error> {
    let t2 = target.as_secs_f64();
    check_riegel_target(t2)?;
    if !k.is_finite() || k <= 0.0 {
        return Err(Error::UnsolvablePowerDuration);
    }
    let t1 = effort.seconds();
    let p2 = effort.power().watts() * (t1 / t2).powf(k);
    Power::new(p2).map_err(|_| Error::UnsolvablePowerDuration)
}

fn predict_power_ftp_percent(effort: Effort, target: Duration) -> Result<Power, Error> {
    let ftp = ftp_from_single_effort(effort)?;
    let pct = ftp_percent_at(target.as_secs_f64())?;
    Power::new(ftp.watts() * pct)
}

/// Predict mean power at `target` duration from a single effort.
///
/// [`PredictionModel::CriticalPower`] on one effort returns
/// [`Error::InsufficientEfforts`] — use [`predict_power_from_efforts`].
pub fn predict_power(
    effort: Effort,
    target: Duration,
    model: PredictionModel,
) -> Result<Power, Error> {
    match model {
        PredictionModel::CriticalPower => Err(Error::InsufficientEfforts),
        PredictionModel::FtpPercent => predict_power_ftp_percent(effort, target),
        PredictionModel::PowerRiegel => riegel_power(effort, target, RIEGEL_POWER_EXPONENT),
    }
}

/// Predict mean power at `target` from one or more efforts.
///
/// For [`PredictionModel::CriticalPower`], fits CP from `efforts` (≥ 2).
/// Other models use `efforts[0]` only.
pub fn predict_power_from_efforts(
    efforts: &[Effort],
    target: Duration,
    model: PredictionModel,
) -> Result<Power, Error> {
    match model {
        PredictionModel::CriticalPower => {
            let fit = critical_power(efforts)?;
            predict_power_from_cp(fit.model, target)
        }
        PredictionModel::FtpPercent | PredictionModel::PowerRiegel => {
            let first = efforts.first().copied().ok_or(Error::InsufficientEfforts)?;
            predict_power(first, target, model)
        }
    }
}

/// Predict duration at `target_power` from a single effort.
///
/// Critical power requires [`predict_duration`] with a multi-effort path via
/// fitting first; this function returns [`Error::InsufficientEfforts`] for
/// [`PredictionModel::CriticalPower`].
pub fn predict_duration(
    effort: Effort,
    target_power: Power,
    model: PredictionModel,
) -> Result<Duration, Error> {
    match model {
        PredictionModel::CriticalPower => Err(Error::InsufficientEfforts),
        PredictionModel::PowerRiegel => {
            // P2 = P1 * (t1/t2)^k  ⇒  t2 = t1 * (P1/P2)^(1/k)
            let ratio = effort.power().watts() / target_power.watts();
            if !(ratio.is_finite() && ratio > 0.0) {
                return Err(Error::UnsolvablePowerDuration);
            }
            let t2 = effort.seconds() * ratio.powf(1.0 / RIEGEL_POWER_EXPONENT);
            check_riegel_target(t2)?;
            Ok(Duration::from_secs_f64(t2))
        }
        PredictionModel::FtpPercent => {
            let ftp = ftp_from_single_effort(effort)?;
            let pct = target_power.watts() / ftp.watts();
            // Invert log-interpolated anchors by bisection on duration.
            let lo = FTP_PCT_ANCHORS[0].0;
            let hi = FTP_PCT_ANCHORS[FTP_PCT_ANCHORS.len() - 1].0;
            let pct_lo = ftp_percent_at(lo)?;
            let pct_hi = ftp_percent_at(hi)?;
            // %FTP decreases with duration; require pct within [pct_hi, pct_lo].
            if pct > pct_lo + 1e-9 || pct < pct_hi - 1e-9 {
                return Err(Error::DurationOutOfModelRange);
            }
            let mut a = lo;
            let mut b = hi;
            for _ in 0..80 {
                let mid = 0.5 * (a + b);
                let p_mid = ftp_percent_at(mid)?;
                if p_mid > pct {
                    a = mid;
                } else {
                    b = mid;
                }
            }
            Ok(Duration::from_secs_f64(0.5 * (a + b)))
        }
    }
}

/// Predict power at `duration` from an already-fitted CP model.
pub fn predict_from_cp(cp: CriticalPower, duration: Duration) -> Result<Power, Error> {
    predict_power_from_cp(cp, duration)
}

#[cfg(test)]
mod tests {
    use super::super::critical_power::{critical_power, predict_duration_from_cp};
    use super::*;

    #[test]
    fn twenty_min_ftp_percent_hour() {
        let twenty = Effort::from_watts_secs(280.0, 1200.0).unwrap();
        // FTP = 266; 60 min = 100% FTP → 266 W
        let hour = predict_power(
            twenty,
            Duration::from_secs(3600),
            PredictionModel::FtpPercent,
        )
        .unwrap();
        assert!((hour.watts() - 266.0).abs() < 0.5);
    }

    #[test]
    fn riegel_power_scales() {
        let twenty = Effort::from_watts_secs(280.0, 1200.0).unwrap();
        let hour = riegel_power(twenty, Duration::from_secs(3600), RIEGEL_POWER_EXPONENT).unwrap();
        let expected = 280.0 * (1200.0_f64 / 3600.0).powf(0.07);
        assert!((hour.watts() - expected).abs() < 1e-9);
    }

    #[test]
    fn critical_power_needs_two() {
        let twenty = Effort::from_watts_secs(280.0, 1200.0).unwrap();
        assert_eq!(
            predict_power(
                twenty,
                Duration::from_secs(3600),
                PredictionModel::CriticalPower
            ),
            Err(Error::InsufficientEfforts)
        );
        let five = Effort::from_watts_secs(340.0, 300.0).unwrap();
        let p = predict_power_from_efforts(
            &[five, twenty],
            Duration::from_secs(3600),
            PredictionModel::CriticalPower,
        )
        .unwrap();
        assert!(p.watts() > 200.0 && p.watts() < 300.0);
    }

    #[test]
    fn predict_duration_at_or_below_cp_via_fit() {
        let five = Effort::from_watts_secs(310.0, 300.0).unwrap();
        let twenty = Effort::from_watts_secs(265.0, 1200.0).unwrap();
        let fit = critical_power(&[five, twenty]).unwrap();
        assert_eq!(
            predict_duration_from_cp(fit.model, fit.model.cp),
            Err(Error::UnsolvablePowerDuration)
        );
    }
}
