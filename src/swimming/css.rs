//! Critical Swim Speed (CSS) and anaerobic distance capacity (ADC).

use super::{Pace, SwimTime};
use crate::Error;

/// Critical Swim Speed and optional anaerobic distance capacity.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Css {
    pub(crate) velocity_m_s: f64,
    /// Anaerobic distance capacity in metres. `None` for T-30 / single-proxy.
    pub(crate) adc_m: Option<f64>,
}

impl Css {
    /// CSS velocity in metres per second.
    pub const fn velocity_m_s(self) -> f64 {
        self.velocity_m_s
    }

    /// CSS pace (seconds per metre inverted from velocity).
    pub fn pace(self) -> Result<Pace, Error> {
        Pace::from_sec_per_meter(1.0 / self.velocity_m_s)
    }

    /// Anaerobic distance capacity in metres, if known.
    pub const fn adc_m(self) -> Option<f64> {
        self.adc_m
    }
}

// CSS is the slope of distance vs time for two maximal swims (Wakayoshi et al., 1992).
// It is effective threshold speed in water, not a laboratory VO2max.

/// CSS and ADC from two maximal trials (Wakayoshi 1992).
///
/// Requires the same course and stroke, `D_long > D_short`, and `T_long > T_short`.
///
/// ```
/// use sportanalytics::swimming::{css_from_trials, Course, Event, Stroke, SwimTime};
///
/// let short = SwimTime::from_secs(Event::M200, Course::Scm, Stroke::Free, 150.0).unwrap();
/// let long = SwimTime::from_secs(Event::M400, Course::Scm, Stroke::Free, 368.0).unwrap();
/// let css = css_from_trials(short, long).unwrap();
/// assert!((css.pace().unwrap().sec_per_100m() - 109.0).abs() < 1e-6);
/// ```
pub fn css_from_trials(short: SwimTime, long: SwimTime) -> Result<Css, Error> {
    if short.course() != long.course() || short.stroke() != long.stroke() {
        return Err(Error::UnrecognizedDistance);
    }
    let d1 = short.distance_meters();
    let d2 = long.distance_meters();
    let t1 = short.seconds();
    let t2 = long.seconds();
    if d2 <= d1 {
        return Err(Error::TrialsSameDistance);
    }
    if t2 <= t1 {
        return Err(Error::TrialsNotOrdered);
    }
    let v = (d2 - d1) / (t2 - t1);
    if !(v.is_finite() && v > 0.0) {
        return Err(Error::UnsolvableCss);
    }
    let adc = d1 - v * t1;
    Ok(Css {
        velocity_m_s: v,
        adc_m: Some(adc),
    })
}

/// CSS from N maximal trials via OLS of `D = ADC + v T` (Wakayoshi slope).
///
/// Needs at least two points with distinct distances. Same course and stroke
/// required across the slice.
pub fn css_from_trials_n(times: &[SwimTime]) -> Result<Css, Error> {
    if times.is_empty() {
        return Err(Error::EmptyRaces);
    }
    if times.len() == 1 {
        return Err(Error::TrialsSameDistance);
    }
    let course = times[0].course();
    let stroke = times[0].stroke();
    for t in times.iter().skip(1) {
        if t.course() != course || t.stroke() != stroke {
            return Err(Error::UnrecognizedDistance);
        }
    }

    let n = times.len() as f64;
    let mut sum_t = 0.0;
    let mut sum_d = 0.0;
    let mut sum_tt = 0.0;
    let mut sum_td = 0.0;
    let mut min_d = f64::INFINITY;
    let mut max_d = 0.0_f64;
    for swim in times {
        let t = swim.seconds();
        let d = swim.distance_meters();
        sum_t += t;
        sum_d += d;
        sum_tt += t * t;
        sum_td += t * d;
        min_d = min_d.min(d);
        max_d = max_d.max(d);
    }
    if max_d <= min_d {
        return Err(Error::TrialsSameDistance);
    }
    let denom = n * sum_tt - sum_t * sum_t;
    if !(denom.is_finite() && denom.abs() > 0.0) {
        return Err(Error::UnsolvableCss);
    }
    let v = (n * sum_td - sum_t * sum_d) / denom;
    if !(v.is_finite() && v > 0.0) {
        return Err(Error::UnsolvableCss);
    }
    let adc = (sum_d - v * sum_t) / n;
    Ok(Css {
        velocity_m_s: v,
        adc_m: Some(adc),
    })
}

/// CSS from a T-30 (distance covered in 30 minutes). ADC is unknown.
///
/// ```
/// use sportanalytics::swimming::css_from_t30;
///
/// let css = css_from_t30(1500.0).unwrap();
/// assert!((css.velocity_m_s() - 1500.0 / 1800.0).abs() < 1e-12);
/// assert!(css.adc_m().is_none());
/// ```
pub fn css_from_t30(distance_m: f64) -> Result<Css, Error> {
    if !(distance_m.is_finite() && distance_m > 0.0) {
        return Err(Error::InvalidDistance);
    }
    let v = distance_m / 1800.0;
    if !(v.is_finite() && v > 0.0) {
        return Err(Error::UnsolvableCss);
    }
    Ok(Css {
        velocity_m_s: v,
        adc_m: None,
    })
}

/// CSS estimate from a maximal ~1500 m swim: `v = D/T`, ADC unknown.
///
/// This is a single-distance proxy, not a Wakayoshi two-trial CSS. The event
/// distance must be within 50 m of 1500 m.
pub fn css_from_1500(time: SwimTime) -> Result<Css, Error> {
    let d = time.distance_meters();
    if (d - 1500.0).abs() > 50.0 {
        return Err(Error::InvalidDistance);
    }
    let v = d / time.seconds();
    if !(v.is_finite() && v > 0.0) {
        return Err(Error::UnsolvableCss);
    }
    Ok(Css {
        velocity_m_s: v,
        adc_m: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swimming::{Course, Event, Stroke};

    fn free_scm(event: Event, secs: f64) -> SwimTime {
        SwimTime::from_secs(event, Course::Scm, Stroke::Free, secs).unwrap()
    }

    #[test]
    fn classic_200_400_identity() {
        // 400m in 368 s, 200m in 150 s → v = 200/218 ≈ 0.917431 m/s
        // pace/100 = (T400 - T200) / 2 = 109 s = 1:49.0
        let short = free_scm(Event::M200, 150.0);
        let long = free_scm(Event::M400, 368.0);
        let css = css_from_trials(short, long).unwrap();
        let expected_v = 200.0 / 218.0;
        assert!((css.velocity_m_s() - expected_v).abs() < 1e-12);
        let pace100 = css.pace().unwrap().sec_per_100m();
        assert!((pace100 - 109.0).abs() < 1e-9);
        assert!((pace100 - (368.0 - 150.0) / 2.0).abs() < 1e-9);
        let adc = css.adc_m().unwrap();
        assert!((adc - (200.0 - expected_v * 150.0)).abs() < 1e-9);
    }

    #[test]
    fn equal_distances_err() {
        let a = free_scm(Event::M200, 150.0);
        let b = free_scm(Event::M200, 160.0);
        assert_eq!(css_from_trials(a, b), Err(Error::TrialsSameDistance));
    }

    #[test]
    fn long_faster_than_short_err() {
        let short = free_scm(Event::M200, 150.0);
        let long = free_scm(Event::M400, 140.0);
        assert_eq!(css_from_trials(short, long), Err(Error::TrialsNotOrdered));
    }

    #[test]
    fn course_mismatch_err() {
        let short = SwimTime::from_secs(Event::M200, Course::Scm, Stroke::Free, 150.0).unwrap();
        let long = SwimTime::from_secs(Event::M400, Course::Lcm, Stroke::Free, 368.0).unwrap();
        assert_eq!(
            css_from_trials(short, long),
            Err(Error::UnrecognizedDistance)
        );
    }

    #[test]
    fn t30_1500_in_30_min() {
        let css = css_from_t30(1500.0).unwrap();
        assert!((css.velocity_m_s() - 0.833_333_333_333_333_4).abs() < 1e-12);
        assert!(css.adc_m().is_none());
    }

    #[test]
    fn ols_two_points_matches_two_trial() {
        let short = free_scm(Event::M200, 150.0);
        let long = free_scm(Event::M400, 368.0);
        let two = css_from_trials(short, long).unwrap();
        let n = css_from_trials_n(&[short, long]).unwrap();
        assert!((two.velocity_m_s() - n.velocity_m_s()).abs() < 1e-12);
        assert!((two.adc_m().unwrap() - n.adc_m().unwrap()).abs() < 1e-12);
    }

    #[test]
    fn css_from_1500_proxy() {
        let t = free_scm(Event::M1500, 1200.0);
        let css = css_from_1500(t).unwrap();
        assert!((css.velocity_m_s() - 1.25).abs() < 1e-12);
        assert!(css.adc_m().is_none());
        let bad = free_scm(Event::M400, 368.0);
        assert_eq!(css_from_1500(bad), Err(Error::InvalidDistance));
    }

    #[test]
    fn empty_and_single_n() {
        assert_eq!(css_from_trials_n(&[]), Err(Error::EmptyRaces));
        let one = free_scm(Event::M400, 368.0);
        assert_eq!(css_from_trials_n(&[one]), Err(Error::TrialsSameDistance));
    }
}
