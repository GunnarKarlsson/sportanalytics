use std::fmt;
use std::time::Duration;

use super::Error;
use super::{Distance, Pace};

/// A single race result: a distance and a positive finish time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaceTime {
    distance: Distance,
    time: Duration,
}

impl RaceTime {
    /// Build a race result from a [`Duration`].
    ///
    /// Returns [`crate::Error::NonPositiveTime`] when `time` is zero.
    pub fn new(distance: Distance, time: Duration) -> Result<Self, Error> {
        if time.as_secs_f64() <= 0.0 {
            return Err(crate::Error::NonPositiveTime.into());
        }
        Ok(Self { distance, time })
    }

    /// Build a race result from hours, minutes, and seconds.
    ///
    /// Minutes and seconds must be `< 60`. Use a larger hour value instead of
    /// overflowing minutes (`1, 30, 0` not `0, 90, 0`).
    ///
    /// Returns [`crate::Error::InvalidHms`] when minutes or seconds are ≥ 60, and
    /// [`crate::Error::NonPositiveTime`] when the total duration is zero.
    ///
    /// ```
    /// use sportanalytics::running::{Distance, RaceTime};
    ///
    /// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
    /// assert_eq!(five.seconds(), 1200.0);
    /// ```
    pub fn from_hms(
        distance: Distance,
        hours: u64,
        minutes: u64,
        seconds: u64,
    ) -> Result<Self, Error> {
        if minutes >= 60 || seconds >= 60 {
            return Err(crate::Error::InvalidHms.into());
        }
        Self::new(
            distance,
            Duration::from_secs(hours * 3600 + minutes * 60 + seconds),
        )
    }

    /// Build a race result from a finish time in seconds.
    ///
    /// Returns [`crate::Error::NonPositiveTime`] when `seconds` is non-finite or not
    /// strictly positive.
    ///
    /// ```
    /// use sportanalytics::running::{Distance, RaceTime};
    ///
    /// let five = RaceTime::from_secs(Distance::FiveK, 1200.0).unwrap();
    /// assert_eq!(five.minutes(), 20.0);
    /// ```
    pub fn from_secs(distance: Distance, seconds: f64) -> Result<Self, Error> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err(crate::Error::NonPositiveTime.into());
        }
        Self::new(distance, Duration::from_secs_f64(seconds))
    }

    /// Build a race result from a distance and an average [`Pace`].
    ///
    /// Finish time is `pace.sec_per_meter() * distance.meters()`.
    ///
    /// ```
    /// use sportanalytics::running::{Distance, Pace, RaceTime};
    ///
    /// let pace = Pace::from_hms_per_km(0, 4, 0).unwrap();
    /// let five = RaceTime::from_pace(Distance::FiveK, pace).unwrap();
    /// assert_eq!(five.seconds(), 1_200.0);
    /// ```
    pub fn from_pace(distance: Distance, pace: Pace) -> Result<Self, Error> {
        Self::from_secs(distance, pace.sec_per_meter() * distance.meters())
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

    /// Average pace over the race distance.
    ///
    /// ```
    /// use sportanalytics::running::{Distance, RaceTime};
    ///
    /// let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
    /// assert_eq!(five.pace().to_string(), "4:00 /km");
    /// ```
    pub fn pace(self) -> Pace {
        Pace::from_sec_per_meter(self.seconds() / self.distance.meters())
            .expect("RaceTime invariants imply a positive finite pace")
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
    fn pace_from_20_min_5k() {
        let race = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
        assert_eq!(race.pace().to_string(), "4:00 /km");
    }

    #[test]
    fn from_pace_builds_finish_time() {
        let pace = Pace::from_hms_per_km(0, 4, 0).unwrap();
        let race = RaceTime::from_pace(Distance::FiveK, pace).unwrap();
        assert_eq!(race.seconds(), 1_200.0);
        assert_eq!(race.pace().sec_per_km(), 240.0);
    }

    #[test]
    fn rejects_zero_and_non_finite_times() {
        assert_eq!(
            RaceTime::from_hms(Distance::TenK, 0, 0, 0),
            Err(crate::Error::NonPositiveTime.into())
        );
        assert_eq!(
            RaceTime::from_secs(Distance::TenK, 0.0),
            Err(crate::Error::NonPositiveTime.into())
        );
        assert_eq!(
            RaceTime::from_secs(Distance::TenK, -1.0),
            Err(crate::Error::NonPositiveTime.into())
        );
        assert_eq!(
            RaceTime::from_secs(Distance::TenK, f64::NAN),
            Err(crate::Error::NonPositiveTime.into())
        );
        assert_eq!(
            RaceTime::from_secs(Distance::TenK, f64::INFINITY),
            Err(crate::Error::NonPositiveTime.into())
        );
        assert_eq!(
            RaceTime::new(Distance::FiveK, Duration::ZERO),
            Err(crate::Error::NonPositiveTime.into())
        );
    }

    #[test]
    fn from_hms_rejects_overflow_minutes_and_seconds() {
        assert_eq!(
            RaceTime::from_hms(Distance::FiveK, 0, 90, 0),
            Err(crate::Error::InvalidHms.into())
        );
        assert_eq!(
            RaceTime::from_hms(Distance::FiveK, 0, 0, 60),
            Err(crate::Error::InvalidHms.into())
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
