//! World Aquatics points.

use super::{Course, Event, Sex, Stroke, SwimTime};
use crate::Error;

// World Aquatics: P = floor(1000 * (B/T)^3). Formula is public.
// Base times B change by year; pass B in or use a generated table — do not hand-edit grids.

/// Lookup table of World Aquatics base times `B` (seconds).
///
/// Full yearly grids are not hand-copied here. Use [`WaPointsTable::empty`] with
/// [`wa_points_with`], or [`WaPointsTable::sample`] for documented fixtures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaPointsTable {
    year: u16,
    rows: &'static [WaBaseRow],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct WaBaseRow {
    event: Event,
    course: Course,
    stroke: Stroke,
    sex: Sex,
    base_secs: f64,
}

impl WaPointsTable {
    /// Empty table: callers must use [`wa_points_with`].
    pub const fn empty() -> Self {
        Self { year: 0, rows: &[] }
    }

    /// Tiny documented sample rows for tests and docs (not a full official year).
    pub const fn sample() -> Self {
        Self {
            year: 0,
            rows: SAMPLE_ROWS,
        }
    }

    /// Table year label (`0` for empty/sample).
    pub const fn year(self) -> u16 {
        self.year
    }

    /// Base time `B` in seconds, if present.
    pub fn base_secs(&self, event: Event, course: Course, stroke: Stroke, sex: Sex) -> Option<f64> {
        self.rows.iter().find_map(|row| {
            if row.event == event && row.course == course && row.stroke == stroke && row.sex == sex
            {
                Some(row.base_secs)
            } else {
                None
            }
        })
    }
}

/// Sample fixture rows (illustrative base times only).
const SAMPLE_ROWS: &[WaBaseRow] = &[
    WaBaseRow {
        event: Event::M100,
        course: Course::Lcm,
        stroke: Stroke::Free,
        sex: Sex::Male,
        base_secs: 46.40,
    },
    WaBaseRow {
        event: Event::M400,
        course: Course::Scm,
        stroke: Stroke::Free,
        sex: Sex::Male,
        base_secs: 220.0,
    },
];

/// World Aquatics points from finish time `T` and base time `B`.
///
/// `P = floor(1000 * (B/T)^3)`.
///
/// ```
/// use sportanalytics::swimming::wa_points_with;
///
/// assert_eq!(wa_points_with(49.0, 46.40).unwrap(), 849);
/// ```
pub fn wa_points_with(time_secs: f64, base_secs: f64) -> Result<u32, Error> {
    if !(time_secs.is_finite() && time_secs > 0.0) {
        return Err(Error::NonPositiveTime);
    }
    if !(base_secs.is_finite() && base_secs > 0.0) {
        return Err(Error::InvalidDistance);
    }
    let ratio = base_secs / time_secs;
    let points = (1000.0 * ratio.powi(3)).floor();
    if !(points.is_finite() && points >= 0.0) {
        return Err(Error::NonPositiveTime);
    }
    Ok(points as u32)
}

/// Invert points to a finish time for base `B`.
///
/// Starts from `T = B / (P/1000).cbrt()`, rounds to hundredths, then walks by
/// 0.01 s to the slowest time that still scores exactly `points` (World
/// Aquatics hundredths procedure).
pub fn time_from_points_with(points: u32, base_secs: f64) -> Result<f64, Error> {
    if points == 0 {
        return Err(Error::NonPositiveTime);
    }
    if !(base_secs.is_finite() && base_secs > 0.0) {
        return Err(Error::InvalidDistance);
    }
    let mut t = base_secs / ((points as f64) / 1000.0).cbrt();
    t = (t * 100.0).round() / 100.0;
    // Faster time ⇒ more points. Walk until we land on exactly `points`.
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 100_000 {
            return Err(Error::NonPositiveTime);
        }
        let p = wa_points_with(t, base_secs)?;
        match p.cmp(&points) {
            std::cmp::Ordering::Equal => break,
            std::cmp::Ordering::Greater => t += 0.01,
            std::cmp::Ordering::Less => t -= 0.01,
        }
        if !(t.is_finite() && t > 0.0) {
            return Err(Error::NonPositiveTime);
        }
        t = (t * 100.0).round() / 100.0;
    }
    // Prefer the slowest hundredth that still scores `points`.
    while wa_points_with(t + 0.01, base_secs).unwrap_or(0) == points {
        t += 0.01;
        t = (t * 100.0).round() / 100.0;
        guard += 1;
        if guard > 100_000 {
            return Err(Error::NonPositiveTime);
        }
    }
    Ok(t)
}

/// World Aquatics points using a base-time table.
///
/// Returns [`Error::UnsupportedWaEvent`] when the table has no row.
pub fn wa_points(time: SwimTime, sex: Sex, table: &WaPointsTable) -> Result<u32, Error> {
    let base = table
        .base_secs(time.event(), time.course(), time.stroke(), sex)
        .ok_or(Error::UnsupportedWaEvent)?;
    wa_points_with(time.seconds(), base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_fixture_849() {
        // B=46.40, T=49.00 → floor(1000*(46.40/49.00)^3) = 849
        assert_eq!(wa_points_with(49.0, 46.40).unwrap(), 849);
    }

    #[test]
    fn rejects_non_positive() {
        assert_eq!(wa_points_with(0.0, 46.40), Err(Error::NonPositiveTime));
        assert_eq!(wa_points_with(49.0, 0.0), Err(Error::InvalidDistance));
        assert_eq!(wa_points_with(-1.0, 46.40), Err(Error::NonPositiveTime));
        assert_eq!(wa_points_with(49.0, -1.0), Err(Error::InvalidDistance));
    }

    #[test]
    fn sample_table_lookup() {
        let t = SwimTime::from_secs(Event::M100, Course::Lcm, Stroke::Free, 49.0).unwrap();
        assert_eq!(
            wa_points(t, Sex::Male, &WaPointsTable::sample()).unwrap(),
            849
        );
        assert_eq!(
            wa_points(t, Sex::Female, &WaPointsTable::sample()),
            Err(Error::UnsupportedWaEvent)
        );
        assert_eq!(
            wa_points(t, Sex::Male, &WaPointsTable::empty()),
            Err(Error::UnsupportedWaEvent)
        );
    }

    #[test]
    fn time_from_points_roundtrip() {
        let t = time_from_points_with(849, 46.40).unwrap();
        assert_eq!(wa_points_with(t, 46.40).unwrap(), 849);
    }
}
