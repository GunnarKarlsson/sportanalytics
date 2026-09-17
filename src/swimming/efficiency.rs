//! Stroke efficiency helpers (SWOLF and distance per stroke).

use crate::Error;

/// SWOLF = seconds for one length + stroke cycles on that length.
///
/// ```
/// use sportanalytics::swimming::swolf;
///
/// assert_eq!(swolf(30.0, 15.0).unwrap(), 45.0);
/// ```
pub fn swolf(length_seconds: f64, stroke_count: f64) -> Result<f64, Error> {
    if !(length_seconds.is_finite() && length_seconds > 0.0) {
        return Err(Error::NonPositiveTime);
    }
    if !(stroke_count.is_finite() && stroke_count > 0.0) {
        return Err(Error::InvalidStrokeCount);
    }
    Ok(length_seconds + stroke_count)
}

/// Distance per stroke in metres.
///
/// ```
/// use sportanalytics::swimming::distance_per_stroke;
///
/// assert_eq!(distance_per_stroke(25.0, 10.0).unwrap(), 2.5);
/// ```
pub fn distance_per_stroke(length_m: f64, stroke_count: f64) -> Result<f64, Error> {
    if !(length_m.is_finite() && length_m > 0.0) {
        return Err(Error::InvalidDistance);
    }
    if !(stroke_count.is_finite() && stroke_count > 0.0) {
        return Err(Error::InvalidStrokeCount);
    }
    Ok(length_m / stroke_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swolf_and_dps() {
        assert_eq!(swolf(32.5, 14.0).unwrap(), 46.5);
        assert!((distance_per_stroke(50.0, 20.0).unwrap() - 2.5).abs() < 1e-12);
    }

    #[test]
    fn rejects_bad_stroke_count() {
        assert_eq!(swolf(30.0, 0.0), Err(Error::InvalidStrokeCount));
        assert_eq!(
            distance_per_stroke(25.0, -1.0),
            Err(Error::InvalidStrokeCount)
        );
        assert_eq!(swolf(0.0, 10.0), Err(Error::NonPositiveTime));
        assert_eq!(distance_per_stroke(0.0, 10.0), Err(Error::InvalidDistance));
    }
}
