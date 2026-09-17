use std::fmt;

use super::{Course, Event, Pace, Stroke};
use crate::Error;

/// A single swim result: event, course, stroke, and a positive finish time.
///
/// Hundredths matter in swimming; time is stored as seconds (`f64`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwimTime {
    event: Event,
    course: Course,
    stroke: Stroke,
    seconds: f64,
}

impl SwimTime {
    /// Build a swim result from a finish time in seconds.
    ///
    /// Returns [`Error::NonPositiveTime`] when `seconds` is non-finite or not
    /// strictly positive.
    pub fn new(event: Event, course: Course, stroke: Stroke, seconds: f64) -> Result<Self, Error> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err(Error::NonPositiveTime);
        }
        Ok(Self {
            event,
            course,
            stroke,
            seconds,
        })
    }

    /// Build from hours, minutes, seconds, and hundredths.
    ///
    /// Minutes and seconds must be `< 60` ([`Error::InvalidHms`]). Hundredths
    /// must be `< 100` ([`Error::InvalidCents`]).
    ///
    /// ```
    /// use sportanalytics::swimming::{Course, Event, Stroke, SwimTime};
    ///
    /// let t = SwimTime::from_hms_cents(
    ///     Event::M100, Course::Scm, Stroke::Free, 0, 1, 2, 45,
    /// ).unwrap();
    /// assert!((t.seconds() - 62.45).abs() < 1e-9);
    /// ```
    pub fn from_hms_cents(
        event: Event,
        course: Course,
        stroke: Stroke,
        hours: u64,
        minutes: u64,
        seconds: u64,
        hundredths: u64,
    ) -> Result<Self, Error> {
        if minutes >= 60 || seconds >= 60 {
            return Err(Error::InvalidHms);
        }
        if hundredths >= 100 {
            return Err(Error::InvalidCents);
        }
        let total = hours as f64 * 3600.0
            + minutes as f64 * 60.0
            + seconds as f64
            + hundredths as f64 / 100.0;
        Self::new(event, course, stroke, total)
    }

    /// Build from a finish time in seconds (alias of [`Self::new`]).
    pub fn from_secs(
        event: Event,
        course: Course,
        stroke: Stroke,
        seconds: f64,
    ) -> Result<Self, Error> {
        Self::new(event, course, stroke, seconds)
    }

    /// Event distance.
    pub const fn event(self) -> Event {
        self.event
    }

    /// Pool course.
    pub const fn course(self) -> Course {
        self.course
    }

    /// Stroke.
    pub const fn stroke(self) -> Stroke {
        self.stroke
    }

    /// Finish time in seconds.
    pub const fn seconds(self) -> f64 {
        self.seconds
    }

    /// Event distance in metres.
    pub fn distance_meters(self) -> f64 {
        self.event.meters()
    }

    /// Average pace over the event distance.
    pub fn pace(self) -> Pace {
        Pace::from_sec_per_meter(self.seconds / self.event.meters())
            .expect("SwimTime invariants imply a positive finite pace")
    }
}

impl fmt::Display for SwimTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.event.label(),
            self.course,
            self.stroke,
            format_swim_hms(self.seconds)
        )
    }
}

/// Format swim time with hundredths (`m:ss.hh` or `h:mm:ss.hh`).
pub(crate) fn format_swim_hms(total_secs: f64) -> String {
    let cents = (total_secs.max(0.0) * 100.0).round() as u64;
    let h = cents / 360_000;
    let rem = cents % 360_000;
    let m = rem / 6_000;
    let s = (rem % 6_000) / 100;
    let c = rem % 100;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}.{c:02}")
    } else {
        format!("{m}:{s:02}.{c:02}")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for SwimTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SwimTime", 4)?;
        state.serialize_field("event", &self.event)?;
        state.serialize_field("course", &self.course)?;
        state.serialize_field("stroke", &self.stroke)?;
        state.serialize_field("seconds", &self.seconds)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SwimTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Helper {
            event: Event,
            course: Course,
            stroke: Stroke,
            seconds: f64,
        }
        let helper = Helper::deserialize(deserializer)?;
        SwimTime::new(helper.event, helper.course, helper.stroke, helper.seconds)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn free_scm(event: Event, secs: f64) -> SwimTime {
        SwimTime::from_secs(event, Course::Scm, Stroke::Free, secs).unwrap()
    }

    #[test]
    fn from_hms_cents_and_from_secs_agree() {
        let a =
            SwimTime::from_hms_cents(Event::M100, Course::Scm, Stroke::Free, 0, 1, 2, 45).unwrap();
        let b = SwimTime::from_secs(Event::M100, Course::Scm, Stroke::Free, 62.45).unwrap();
        assert!((a.seconds() - b.seconds()).abs() < 1e-9);
        assert_eq!(a.event(), Event::M100);
        assert_eq!(a.course(), Course::Scm);
        assert_eq!(a.stroke(), Stroke::Free);
    }

    #[test]
    fn from_hms_cents_rejects_overflow() {
        assert_eq!(
            SwimTime::from_hms_cents(Event::M100, Course::Scm, Stroke::Free, 0, 90, 0, 0),
            Err(Error::InvalidHms)
        );
        assert_eq!(
            SwimTime::from_hms_cents(Event::M100, Course::Scm, Stroke::Free, 0, 0, 60, 0),
            Err(Error::InvalidHms)
        );
        assert_eq!(
            SwimTime::from_hms_cents(Event::M100, Course::Scm, Stroke::Free, 0, 1, 0, 100),
            Err(Error::InvalidCents)
        );
    }

    #[test]
    fn rejects_non_positive_time() {
        assert_eq!(
            SwimTime::from_secs(Event::M100, Course::Lcm, Stroke::Free, 0.0),
            Err(Error::NonPositiveTime)
        );
        assert_eq!(
            SwimTime::from_secs(Event::M100, Course::Lcm, Stroke::Free, f64::NAN),
            Err(Error::NonPositiveTime)
        );
    }

    #[test]
    fn display_includes_course_and_stroke() {
        let t = free_scm(Event::M100, 62.45);
        assert_eq!(t.to_string(), "100m SCM Free 1:02.45");
    }

    #[test]
    fn pace_from_100m() {
        let t = free_scm(Event::M100, 100.0);
        assert!((t.pace().sec_per_100m() - 100.0).abs() < 1e-12);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip() {
        let t = free_scm(Event::M200, 150.0);
        let back: SwimTime = serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap();
        assert_eq!(back, t);
    }
}
