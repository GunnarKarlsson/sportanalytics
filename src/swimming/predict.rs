//! Race-time prediction for swimming.

use std::fmt;

use super::css::Css;
use super::time::format_swim_hms;
use super::{Course, Event, Stroke, SwimTime};
use crate::Error;

// RIEGEL_EXPONENT is the swimming factor (~1.03) from Riegel 1981 American Scientist.
// Do not use the running constant 1.06 here.

/// Pete Riegel 1981 *American Scientist* swimming fatigue factor (men ~1.030,
/// women ~1.033). Default is the rounded published swimming value, not the
/// running 1.06.
pub const RIEGEL_EXPONENT: f64 = 1.03;

/// Which scaling model to use for [`predict_times`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PredictionModel {
    /// `T = (D − ADC) / v_css` when ADC known and `D > ADC`; else `D / v_css`.
    CssAdc,
    /// `T2 = T1 * (D2/D1)^k` with `k =` [`RIEGEL_EXPONENT`].
    Riegel,
}

/// Predicted swim times from a source result and optional CSS.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PredictedTimes {
    /// Model that produced these times.
    pub model: PredictionModel,
    /// Swim that was scaled (or a synthetic source from [`predict_from_css`]).
    pub source: SwimTime,
    /// CSS used for [`PredictionModel::CssAdc`], if available.
    pub css: Option<Css>,
    /// Riegel `k` when `model == Riegel`; otherwise `None`.
    pub riegel_exponent: Option<f64>,
}

impl PredictedTimes {
    /// Predicted finish time in seconds for `event`.
    ///
    /// Same course is assumed; this crate does not convert LCM ↔ SCM ↔ SCY.
    pub fn seconds(&self, event: Event) -> Result<f64, Error> {
        match self.model {
            PredictionModel::Riegel => {
                let k = self.riegel_exponent.unwrap_or(RIEGEL_EXPONENT);
                riegel(self.source, event, k)
            }
            PredictionModel::CssAdc => {
                let css = match self.css {
                    Some(c) => c,
                    None => {
                        // Constant-pace fallback: v = D/T, ADC unknown.
                        let v = self.source.distance_meters() / self.source.seconds();
                        if !(v.is_finite() && v > 0.0) {
                            return Err(Error::UnsolvableCss);
                        }
                        Css {
                            velocity_m_s: v,
                            adc_m: None,
                        }
                    }
                };
                css_adc_time(css, event.meters())
            }
        }
    }

    /// Predicted finish time formatted with hundredths.
    pub fn formatted(&self, event: Event) -> Result<String, Error> {
        Ok(format_swim_hms(self.seconds(event)?))
    }
}

impl fmt::Display for PredictedTimes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} from {}", self.model, self.source)
    }
}

fn css_adc_time(css: Css, distance_m: f64) -> Result<f64, Error> {
    if !(distance_m.is_finite() && distance_m > 0.0) {
        return Err(Error::InvalidDistance);
    }
    let v = css.velocity_m_s();
    if !(v.is_finite() && v > 0.0) {
        return Err(Error::UnsolvableCss);
    }
    let t = match css.adc_m() {
        Some(adc) if distance_m > adc => (distance_m - adc) / v,
        _ => distance_m / v,
    };
    if !(t.is_finite() && t > 0.0) {
        return Err(Error::NonPositiveTime);
    }
    Ok(t)
}

/// Riegel power-law prediction with exponent `k`.
///
/// ```
/// use sportanalytics::swimming::{riegel, Course, Event, Stroke, SwimTime, RIEGEL_EXPONENT};
///
/// let src = SwimTime::from_secs(Event::M400, Course::Scm, Stroke::Free, 300.0).unwrap();
/// let t = riegel(src, Event::M400, RIEGEL_EXPONENT).unwrap();
/// assert!((t - 300.0).abs() < 1e-12);
/// ```
pub fn riegel(source: SwimTime, target: Event, k: f64) -> Result<f64, Error> {
    if !(k.is_finite()) {
        return Err(Error::InvalidPace);
    }
    let d1 = source.distance_meters();
    let d2 = target.meters();
    if !(d1 > 0.0 && d2 > 0.0) {
        return Err(Error::InvalidDistance);
    }
    let t = source.seconds() * (d2 / d1).powf(k);
    if !(t.is_finite() && t > 0.0) {
        return Err(Error::NonPositiveTime);
    }
    Ok(t)
}

/// Predict times from a single known swim.
///
/// [`PredictionModel::CssAdc`] on one swim uses constant-pace (`v = D/T`, no ADC).
/// Prefer [`predict_from_css`] with a two-trial CSS when available.
///
/// ```
/// use sportanalytics::swimming::{predict_times, Course, Event, PredictionModel, Stroke, SwimTime};
///
/// let t400 = SwimTime::from_secs(Event::M400, Course::Scm, Stroke::Free, 320.0).unwrap();
/// let pred = predict_times(t400, PredictionModel::Riegel).unwrap();
/// let t800 = pred.seconds(Event::M800).unwrap();
/// assert!((t800 - 320.0 * 2_f64.powf(1.03)).abs() < 1e-9);
/// ```
pub fn predict_times(source: SwimTime, model: PredictionModel) -> Result<PredictedTimes, Error> {
    Ok(PredictedTimes {
        model,
        source,
        css: None,
        riegel_exponent: match model {
            PredictionModel::Riegel => Some(RIEGEL_EXPONENT),
            PredictionModel::CssAdc => None,
        },
    })
}

/// Predict times from a known CSS (preferred path for [`PredictionModel::CssAdc`]).
///
/// Builds a synthetic source at 400 m (or the CSS pace × 400 m) on the given
/// course and stroke for Riegel fallbacks and display.
pub fn predict_from_css(
    css: Css,
    course: Course,
    stroke: Stroke,
    model: PredictionModel,
) -> Result<PredictedTimes, Error> {
    let t400 = css_adc_time(css, Event::M400.meters())?;
    let source = SwimTime::from_secs(Event::M400, course, stroke, t400)?;
    Ok(PredictedTimes {
        model,
        source,
        css: Some(css),
        riegel_exponent: match model {
            PredictionModel::Riegel => Some(RIEGEL_EXPONENT),
            PredictionModel::CssAdc => None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swimming::{css_from_trials, Course, Event, Stroke};

    fn free_scm(event: Event, secs: f64) -> SwimTime {
        SwimTime::from_secs(event, Course::Scm, Stroke::Free, secs).unwrap()
    }

    #[test]
    fn riegel_identity_and_double_distance() {
        let src = free_scm(Event::M400, 300.0);
        assert!((riegel(src, Event::M400, RIEGEL_EXPONENT).unwrap() - 300.0).abs() < 1e-12);
        let doubled = riegel(src, Event::M800, RIEGEL_EXPONENT).unwrap();
        assert!((doubled - 300.0 * 2_f64.powf(1.03)).abs() < 1e-9);
    }

    #[test]
    fn css_adc_reconstructs_trial_time() {
        let short = free_scm(Event::M200, 150.0);
        let long = free_scm(Event::M400, 368.0);
        let css = css_from_trials(short, long).unwrap();
        let pred =
            predict_from_css(css, Course::Scm, Stroke::Free, PredictionModel::CssAdc).unwrap();
        let t200 = pred.seconds(Event::M200).unwrap();
        let t400 = pred.seconds(Event::M400).unwrap();
        assert!((t200 - 150.0).abs() < 1e-6);
        assert!((t400 - 368.0).abs() < 1e-6);
    }

    #[test]
    fn css_adc_single_swim_constant_pace() {
        let src = free_scm(Event::M400, 400.0);
        let pred = predict_times(src, PredictionModel::CssAdc).unwrap();
        let t200 = pred.seconds(Event::M200).unwrap();
        assert!((t200 - 200.0).abs() < 1e-9);
    }
}
