use std::fmt;
use std::time::Duration;

use super::Distance;
use crate::Error;

/// A single race result: a distance and a positive finish time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaceTime {
    distance: Distance,
    time: Duration,
}

impl RaceTime {
    /// Build a race result from a [`Duration`].
    ///
    /// Returns [`Error::NonPositiveTime`] when `time` is zero.
    pub fn new(distance: Distance, time: Duration) -> Result<Self, Error> {
        if time.as_secs_f64() <= 0.0 {
            return Err(Error::NonPositiveTime);
        }
        Ok(Self { distance, time })
    }

    /// Build a race result from hours, minutes, and seconds.
    ///
    /// Minutes and seconds must be `< 60`. Use a larger hour value instead of
    /// overflowing minutes (`1, 30, 0` not `0, 90, 0`).
    ///
    /// Returns [`Error::InvalidHms`] when minutes or seconds are ≥ 60, and
    /// [`Error::NonPositiveTime`] when the total duration is zero.
    pub fn from_hms(
        distance: Distance,
        hours: u64,
        minutes: u64,
        seconds: u64,
    ) -> Result<Self, Error> {
        if minutes >= 60 || seconds >= 60 {
            return Err(Error::InvalidHms);
        }
        Self::new(
            distance,
            Duration::from_secs(hours * 3600 + minutes * 60 + seconds),
        )
    }

    /// Build a race result from a finish time in seconds.
    ///
    /// Returns [`Error::NonPositiveTime`] when `seconds` is non-finite or not
    /// strictly positive.
    pub fn from_secs(distance: Distance, seconds: f64) -> Result<Self, Error> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err(Error::NonPositiveTime);
        }
        Self::new(distance, Duration::from_secs_f64(seconds))
    }

    /// Race distance.
    pub const fn distance(self) -> Distance {
        self.distance
    }

    /// Finish time as a [`Duration`].
    pub const fn time(self) -> Duration {
        self.time
    }

    /// Finish time in seconds.
    pub fn seconds(self) -> f64 {
        self.time.as_secs_f64()
    }

    /// Finish time in minutes.
    pub fn minutes(self) -> f64 {
        self.seconds() / 60.0
    }

    /// Average velocity in metres per minute (Daniels units).
    pub fn velocity_m_per_min(self) -> f64 {
        self.distance.meters() / self.minutes()
    }
}

impl fmt::Display for RaceTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.distance.label(),
            format_hms(self.seconds())
        )
    }
}

pub(crate) fn format_hms(total_secs: f64) -> String {
    let total = total_secs.max(0.0).round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RaceTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("RaceTime", 2)?;
        state.serialize_field("distance", &self.distance)?;
        state.serialize_field("seconds", &self.seconds())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RaceTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Helper {
            distance: Distance,
            seconds: f64,
        }
        let helper = Helper::deserialize(deserializer)?;
        RaceTime::from_secs(helper.distance, helper.seconds).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hms_and_from_secs_agree() {
        let a = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let b = RaceTime::from_secs(Distance::FiveK, 1200.0).unwrap();
        assert_eq!(a.seconds(), b.seconds());
        assert_eq!(a.distance(), Distance::FiveK);
        assert_eq!(a.time(), Duration::from_secs(1200));
        assert_eq!(a.minutes(), 20.0);
    }

    #[test]
    fn velocity_for_20_min_5k() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        assert!((race.velocity_m_per_min() - 250.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_zero_and_non_finite_times() {
        assert_eq!(
            RaceTime::from_hms(Distance::TenK, 0, 0, 0),
            Err(Error::NonPositiveTime)
        );
        assert_eq!(
            RaceTime::from_secs(Distance::TenK, 0.0),
            Err(Error::NonPositiveTime)
        );
        assert_eq!(
            RaceTime::from_secs(Distance::TenK, -1.0),
            Err(Error::NonPositiveTime)
        );
        assert_eq!(
            RaceTime::from_secs(Distance::TenK, f64::NAN),
            Err(Error::NonPositiveTime)
        );
        assert_eq!(
            RaceTime::from_secs(Distance::TenK, f64::INFINITY),
            Err(Error::NonPositiveTime)
        );
        assert_eq!(
            RaceTime::new(Distance::FiveK, Duration::ZERO),
            Err(Error::NonPositiveTime)
        );
    }

    #[test]
    fn from_hms_rejects_overflow_minutes_and_seconds() {
        assert_eq!(
            RaceTime::from_hms(Distance::FiveK, 0, 90, 0),
            Err(Error::InvalidHms)
        );
        assert_eq!(
            RaceTime::from_hms(Distance::FiveK, 0, 0, 60),
            Err(Error::InvalidHms)
        );
        let ninety = RaceTime::from_hms(Distance::HalfMarathon, 1, 30, 0).unwrap();
        assert_eq!(ninety.seconds(), 5400.0);
    }

    #[test]
    fn display_uses_short_and_long_formats() {
        let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 5).unwrap();
        assert_eq!(five.to_string(), "5K 20:05");
        let marathon = RaceTime::from_hms(Distance::Marathon, 2, 30, 0).unwrap();
        assert_eq!(marathon.to_string(), "FM 2:30:00");
    }

    #[test]
    fn format_hms_rounds_and_drops_zero_hours() {
        assert_eq!(format_hms(59.4), "0:59");
        assert_eq!(format_hms(59.6), "1:00");
        assert_eq!(format_hms(3600.0), "1:00:00");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip_and_rejects_zero() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        let back: RaceTime = serde_json::from_str(&serde_json::to_string(&race).unwrap()).unwrap();
        assert_eq!(back, race);
        assert!(serde_json::from_str::<RaceTime>(r#"{"distance":"FiveK","seconds":0}"#).is_err());
    }
}
