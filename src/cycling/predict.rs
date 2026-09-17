//! Power–duration prediction.
//!
//! * [`PredictionModel::CriticalPower`] — invert two-parameter CP + W′ (needs ≥ 2 efforts).
//! * [`PredictionModel::FtpPercent`] — single effort → FTP via protocol window, then %FTP(t).
//! * [`PredictionModel::PowerRiegel`] — `P2 = P1 * (t1/t2)^k` with default `k = 0.07`.
//!
//! Prefer [`predict_power`] / [`predict_duration`] plus the CP-specific
//! [`crate::cycling::critical_power()`] / [`crate::cycling::predict_power_from_cp`] pair.

use std::time::Duration;

use super::critical_power::{critical_power, predict_power_from_cp};
use super::effort::{Effort, Power};
use super::ftp::{ftp_from_map, ftp_from_protocol, Ftp, FtpProtocol};
use super::Error;

/// Default power-side Riegel exponent (`k = 0.07`).
pub const RIEGEL_POWER_EXPONENT: f64 = 0.07;

/// Which model to use for [`predict_power`] / [`predict_duration`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PredictionModel {
    /// Two-parameter CP + W′ (requires ≥ 2 efforts).
    CriticalPower,
    /// Scale a typical %FTP-by-duration curve from one effort in a protocol window.
    FtpPercent,
    /// Power Riegel: `P2 = P1 * (t1/t2)^k` with default `k = 0.07`.
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

/// Map a single effort to FTP only when duration sits in a protocol window.
///
/// Windows (no overlap):
/// - MAP: **3–6 min exclusive of 6** → `FTP = P / 1.20`
/// - EightMin: **6–10 min** → `FTP = 0.90 × P`
/// - TwentyMin: 15–25 min → `FTP = 0.95 × P`
/// - SixtyMin: 45–75 min → `FTP = P`
///
/// Durations outside those windows return [`Error::DurationOutOfModelRange`]
/// (no nearest-neighbour heuristics). Explicit [`FtpProtocol::RampMap`] still
/// accepts 3–8 min when calling [`crate::cycling::ftp_from_protocol`] directly.
fn ftp_from_single_effort(effort: Effort) -> Result<Ftp, Error> {
    let t = effort.seconds();
    if (15.0 * 60.0..=25.0 * 60.0).contains(&t) {
        return ftp_from_protocol(effort, FtpProtocol::TwentyMin);
    }
    if (45.0 * 60.0..=75.0 * 60.0).contains(&t) {
        return ftp_from_protocol(effort, FtpProtocol::SixtyMin);
    }
    if (6.0 * 60.0..=10.0 * 60.0).contains(&t) {
        return ftp_from_protocol(effort, FtpProtocol::EightMin);
    }
    // MAP / ~5 min: [3, 6) min — does not steal the EightMin band.
    if (3.0 * 60.0..6.0 * 60.0).contains(&t) {
        return ftp_from_map(effort.power());
    }
    Err(Error::DurationOutOfModelRange)
}

fn check_riegel_target(seconds: f64) -> Result<(), Error> {
    if (RIEGEL_TARGET_LO..=RIEGEL_TARGET_HI).contains(&seconds) {
        Ok(())
    } else {
        Err(Error::DurationOutOfModelRange)
    }
}

fn riegel_power(effort: Effort, target: Duration, k: f64) -> Result<Power, Error> {
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

/// Predict mean power at `target` duration from one or more maximal efforts.
///
/// * [`PredictionModel::CriticalPower`] — fits CP from `efforts` (≥ 2). Hour
///   predictions from short+medium pairs are model output, not true FTP.
/// * [`PredictionModel::FtpPercent`] / [`PredictionModel::PowerRiegel`] — use
///   `efforts[0]` only; empty slice → [`Error::InsufficientEfforts`].
///
/// ```
/// use std::time::Duration;
/// use sportanalytics::cycling::{predict_power, Effort, PredictionModel};
///
/// let twenty = Effort::from_watts_secs(280.0, 1200.0).unwrap();
/// let hour = predict_power(
///     &[twenty],
///     Duration::from_secs(3600),
///     PredictionModel::FtpPercent,
/// )
/// .unwrap();
/// assert!((hour.watts() - 266.0).abs() < 0.5);
/// ```
pub fn predict_power(
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
            match model {
                PredictionModel::FtpPercent => predict_power_ftp_percent(first, target),
                PredictionModel::PowerRiegel => riegel_power(first, target, RIEGEL_POWER_EXPONENT),
                PredictionModel::CriticalPower => unreachable!(),
            }
        }
    }
}

/// Predict duration at `target_power` from a single effort.
///
/// [`PredictionModel::CriticalPower`] returns [`Error::InsufficientEfforts`] —
/// fit with [`crate::cycling::critical_power()`] then use
/// [`crate::cycling::predict_duration_from_cp`].
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

#[cfg(test)]
mod tests {
    use super::super::critical_power::{critical_power, predict_duration_from_cp};
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cycling")
    }

    #[test]
    fn fixture_predict_power() {
        let text = fs::read_to_string(fixtures_dir().join("predict.csv")).unwrap();
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }
            let c: Vec<_> = line.split(',').collect();
            let model = match c[0] {
                "ftp_percent" => PredictionModel::FtpPercent,
                "riegel" => PredictionModel::PowerRiegel,
                "cp" => PredictionModel::CriticalPower,
                other => panic!("unknown model {other}"),
            };
            let watts: f64 = c[1].parse().unwrap();
            let seconds: f64 = c[2].parse().unwrap();
            let target_s: f64 = c[3].parse().unwrap();
            let expected: f64 = c[4].parse().unwrap();
            let effort = Effort::from_watts_secs(watts, seconds).unwrap();
            let efforts = if model == PredictionModel::CriticalPower {
                // second effort in cols 5,6 when present
                let w2: f64 = c[5].parse().unwrap();
                let s2: f64 = c[6].parse().unwrap();
                vec![effort, Effort::from_watts_secs(w2, s2).unwrap()]
            } else {
                vec![effort]
            };
            let p = predict_power(&efforts, Duration::from_secs_f64(target_s), model).unwrap();
            assert!(
                (p.watts() - expected).abs() < 0.5,
                "row {i}: got {} want {expected}",
                p.watts()
            );
        }
    }

    #[test]
    fn ftp_percent_rejects_off_window_effort() {
        let twelve = Effort::from_watts_secs(290.0, 12.0 * 60.0).unwrap();
        assert_eq!(
            predict_power(
                &[twelve],
                Duration::from_secs(3600),
                PredictionModel::FtpPercent
            ),
            Err(Error::DurationOutOfModelRange)
        );
        let thirty_five = Effort::from_watts_secs(270.0, 35.0 * 60.0).unwrap();
        assert_eq!(
            predict_power(
                &[thirty_five],
                Duration::from_secs(3600),
                PredictionModel::FtpPercent
            ),
            Err(Error::DurationOutOfModelRange)
        );
    }

    #[test]
    fn eight_min_ftp_percent_matches_eight_min_protocol_not_map() {
        use super::super::ftp::{ftp_from_protocol, FtpProtocol};
        let eight = Effort::from_watts_secs(300.0, 480.0).unwrap();
        let via_protocol = ftp_from_protocol(eight, FtpProtocol::EightMin).unwrap();
        assert!((via_protocol.watts() - 270.0).abs() < 1e-9);
        let hour = predict_power(
            &[eight],
            Duration::from_secs(3600),
            PredictionModel::FtpPercent,
        )
        .unwrap();
        // Must be 270 W (0.90×P), not 250 W (P/1.20 MAP path).
        assert!((hour.watts() - 270.0).abs() < 0.5);
    }

    #[test]
    fn five_min_still_uses_map_not_eight_min() {
        let five = Effort::from_watts_secs(300.0, 300.0).unwrap();
        let hour = predict_power(
            &[five],
            Duration::from_secs(3600),
            PredictionModel::FtpPercent,
        )
        .unwrap();
        // MAP: FTP = 300/1.20 = 250 → hour at 100% FTP.
        assert!((hour.watts() - 250.0).abs() < 0.5);
    }

    #[test]
    fn riegel_power_scales() {
        let twenty = Effort::from_watts_secs(280.0, 1200.0).unwrap();
        let hour = predict_power(
            &[twenty],
            Duration::from_secs(3600),
            PredictionModel::PowerRiegel,
        )
        .unwrap();
        let expected = 280.0 * (1200.0_f64 / 3600.0).powf(0.07);
        assert!((hour.watts() - expected).abs() < 1e-9);
    }

    #[test]
    fn critical_power_needs_two() {
        let twenty = Effort::from_watts_secs(280.0, 1200.0).unwrap();
        assert_eq!(
            predict_power(
                &[twenty],
                Duration::from_secs(3600),
                PredictionModel::CriticalPower
            ),
            Err(Error::InsufficientEfforts)
        );
        let five = Effort::from_watts_secs(340.0, 300.0).unwrap();
        let p = predict_power(
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

    #[test]
    fn empty_efforts_insufficient() {
        assert_eq!(
            predict_power(&[], Duration::from_secs(3600), PredictionModel::FtpPercent),
            Err(Error::InsufficientEfforts)
        );
    }
}
